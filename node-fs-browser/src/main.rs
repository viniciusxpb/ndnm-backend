// node-fs-browser/src/main.rs
use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};
// FIX Warning: Removido walkdir::DirEntry (o compilador já detectou que era inútil)
use std::{path::Path, time::SystemTime};
use chrono::{DateTime, Utc};
use walkdir;

// --- Estruturas de Comunicação (Input/Output) ---

#[derive(Debug, Deserialize)]
pub struct Input {
    // O caminho que o frontend quer explorar (ex: C:\)
    path: String,
}

#[derive(Debug, Serialize, Clone)] 
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub path: String, // Caminho completo para o próximo clique
    pub modified: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub current_path: String,
    pub entries: Vec<DirectoryEntry>,
}

// --- Lógica de Negócio ---
fn get_entries(path_str: &str) -> Result<Vec<DirectoryEntry>, AppError> {
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

        // Ignoramos arquivos/pastas ocultas, como .git
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
            path: parent.to_string_lossy().into_owned(),
            modified: Utc::now(),
        });
    }
    
    // Ordena Pastas primeiro
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    
    Ok(entries)
}

// --- Implementação do Node (para expor a lógica) ---
#[derive(Default)]
pub struct FsBrowserNode;

#[async_trait]
impl Node for FsBrowserNode {
    type Input = Input;
    type Output = Output;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        if input.path.is_empty() {
            return Err(AppError::bad("O campo 'path' não pode ser vazio"));
        }
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        let path_clone = input.path.clone();

        let entries = tokio::task::spawn_blocking(move || {
            get_entries(&path_clone)
        })
        .await
        .map_err(|_| AppError::Internal)?
        ?;

        Ok(Output {
            current_path: input.path,
            entries,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        FsBrowserNode::default(),
        "node-fs-browser",
        "Node de serviço para navegação no sistema de arquivos",
        env!("CARGO_MANIFEST_DIR"),
    )
    .await
}