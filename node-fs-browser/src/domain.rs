// viniciusxpb/ndnm-backend/ndnm-backend-c893a1ebc17c6070ecb4b86d83dbca22839369a/node-fs-browser/src/domain.rs
use ndnm_core::AppError;
use serde::{Serialize};
use std::{path::Path, time::SystemTime};
use chrono::{DateTime, Utc};
use walkdir;

#[derive(Debug, Serialize, Clone)] 
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub path: String, // Caminho completo para o próximo clique
    pub modified: DateTime<Utc>,
}

pub fn get_entries(path_str: &str) -> Result<Vec<DirectoryEntry>, AppError> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(AppError::bad(format!("Caminho não existe: {}", path_str)));
    }
    if !path.is_dir() {
        return Err(AppError::bad(format!("Caminho não é um diretório: {}", path_str)));
    }
    
    let mut entries = Vec::new();
    
    // Usamos WalkDir para varrer apenas o primeiro nível (max_depth(1))
    for entry_result in walkdir::WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry_result.map_err(|e| {
            AppError::bad(format!("Falha ao ler entrada: {}", e))
        })?;

        // Ignoramos arquivos/pastas ocultas
        if entry.file_name().to_string_lossy().starts_with('.') { continue; }
        
        let metadata = entry.metadata().map_err(|e| 
            AppError::bad(format!("Falha ao ler metadados: {}", e))
        )?;

        let modified: DateTime<Utc> = metadata.modified()
            .unwrap_or_else(|_| SystemTime::now()) 
            .into();
        
        entries.push(DirectoryEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: metadata.is_dir(),
            path: entry.path().to_string_lossy().into_owned(),
            modified,
        });
    }

    // Opcional: Adiciona '..' (Up Directory)
    if let Some(parent) = path.parent() {
         entries.push(DirectoryEntry {
            name: "..".to_string(),
            is_dir: true,
            // Certificamos que o path seja o do pai para a navegação funcionar
            path: parent.to_string_lossy().into_owned(),
            modified: Utc::now(),
        });
    }
    
    // Ordena Pastas primeiro
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    
    Ok(entries)
}