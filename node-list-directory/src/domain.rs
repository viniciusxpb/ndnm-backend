// node-list-directory/src/domain.rs
//! Regra de negócio: listar o conteúdo de um diretório.

use ndnm_core::AppError;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Estrutura que representa uma entrada no diretório (arquivo ou pasta)
#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

/// A função principal da nossa lógica de negócio.
pub fn list_directory(path_str: &str) -> Result<Vec<DirectoryEntry>, AppError> {
    let path = Path::new(path_str);

    if !path.exists() {
        return Err(AppError::bad(format!("Caminho não existe: {}", path_str)));
    }

    if !path.is_dir() {
        return Err(AppError::bad(format!(
            "Caminho não é um diretório: {}",
            path_str
        )));
    }

    let mut entries = Vec::new();

    // Itera sobre as entradas do diretório
    let read_dir = fs::read_dir(path).map_err(|e| {
        AppError::bad(format!("Falha ao ler diretório {}: {}", path_str, e))
    })?;

    for entry_result in read_dir {
        let entry = entry_result.map_err(|e| {
            AppError::bad(format!(
                "Falha ao ler entrada no diretório {}: {}",
                path_str, e
            ))
        })?;

        let file_name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry
            .metadata()
            .map_err(|e| AppError::bad(format!("Falha ao ler metadados de {}: {}", file_name, e)))?;

        let is_dir = metadata.is_dir();
        
        // Se for diretório, tamanho é 0. Se for arquivo, pega o tamanho.
        let size_bytes = if is_dir { 0 } else { metadata.len() };

        entries.push(DirectoryEntry {
            name: file_name,
            is_dir,
            size_bytes,
        });
    }

    Ok(entries)
}

// Testes unitários para a nossa lógica!
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    // Helper para criar um ambiente de teste
    fn setup_test_dir(dir_name: &str) -> String {
        let test_dir = std::env::temp_dir().join(dir_name);
        // Limpa se já existir
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir_all(&test_dir).unwrap();

        // Cria um subdiretório
        fs::create_dir(test_dir.join("subfolder")).unwrap();
        // Cria um arquivo
        let mut file = fs::File::create(test_dir.join("file.txt")).unwrap();
        file.write_all(b"hello").unwrap(); // 5 bytes

        test_dir.to_string_lossy().to_string()
    }

    #[test]
    fn test_list_directory_ok() {
        let dir = setup_test_dir("test_list_ok");
        let mut entries = list_directory(&dir).unwrap();

        // Ordena pra garantir a ordem do teste
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(entries.len(), 2);
        
        assert_eq!(entries[0].name, "file.txt");
        assert_eq!(entries[0].is_dir, false);
        assert_eq!(entries[0].size_bytes, 5);
        
        assert_eq!(entries[1].name, "subfolder");
        assert_eq!(entries[1].is_dir, true);
        assert_eq!(entries[1].size_bytes, 0);

        // Limpa
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_list_directory_not_found() {
        let result = list_directory("./caminho-que-nao-existe-12345");
        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Caminho não existe"));
        } else {
            panic!("Esperava AppError::BadRequest");
        }
    }
}