// node-list-directory/src/main.rs
mod domain;

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};

// Importa nossa estrutura de dados do módulo de domínio
use domain::DirectoryEntry;

/// O JSON que o node espera receber
#[derive(Debug, Deserialize)]
pub struct Input {
    path: String,
}

/// O JSON que o node vai responder
#[derive(Debug, Serialize)]
pub struct Output {
    /// O caminho que foi solicitado
    path: String,
    /// A lista de entradas encontradas
    entries: Vec<DirectoryEntry>,
}

#[derive(Default)]
pub struct ListDirectoryNode;

#[async_trait]
impl Node for ListDirectoryNode {
    type Input = Input;
    type Output = Output;

    /// Validação rápida e barata.
    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        if input.path.is_empty() {
            return Err(AppError::bad("O campo 'path' não pode ser vazio"));
        }
        
        // **Visão Além do Alcance (Segurança)**:
        // A gente não quer que o usuário possa pedir "../../../../etc/passwd"
        // Isso é uma verificação simples contra "Path Traversal".
        if input.path.contains("..") {
            return Err(AppError::bad("Path traversal não é permitido. Use caminhos absolutos ou relativos para subpastas."));
        }
        Ok(())
    }

    /// Onde a mágica acontece (de forma assíncrona)
    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        // Clona o path para mover para a thread de bloqueio
        let path_clone = input.path.clone();

        // **A MÁGICA DE PERFORMANCE!**
        // A gente pede pro Tokio rodar nossa função `domain::list_directory`
        // (que é bloqueante) em outra thread.
        // O `.await` aqui espera essa thread terminar, sem travar o servidor.
        let entries = tokio::task::spawn_blocking(move || {
            domain::list_directory(&path_clone)
        })
        .await
        .map_err(|_join_error| AppError::Internal)? // Erro se a thread panicar
        ?; // Erro se o `list_directory` retornar um AppError

        // Se deu tudo certo, monta a resposta
        Ok(Output {
            path: input.path,
            entries,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        ListDirectoryNode::default(),
        "node-list-directory",
        "Node que lista o conteúdo de um diretório no servidor",
        env!("CARGO_MANIFEST_DIR"),
    )
    .await
}