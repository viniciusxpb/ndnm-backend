// node-fs-browser/src/main.rs
mod domain;

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};
use domain::DirectoryEntry;

// NOVOS IMPORTS PARA SERVIDOR MANUAL E CORS
use ndnm_core::{router, load_config};
use tower_http::cors::CorsLayer;
use clap::{FromArgMatches, Parser};
use std::net::SocketAddr;
use axum;
use tokio::net::TcpListener;

// --- Estruturas de Comunicação (Input/Output) ---

#[derive(Debug, Deserialize)]
pub struct Input {
    // PADRONIZADO: Recebe o valor do input field como 'value'
    value: String,
}

#[derive(Debug, Serialize)]
pub struct Output {
    // Mantém o path original para referência
    pub current_path: String,
    // A lista de entradas que o frontend usará para gerar os handles
    pub entries: Vec<DirectoryEntry>,
}

// --- Implementação do Node ---
#[derive(Default)]
pub struct FsBrowserNode;

#[async_trait]
impl Node for FsBrowserNode {
    type Input = Input;
    type Output = Output;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        // Validação usa input.value
        if input.value.is_empty() {
            return Err(AppError::bad("O campo 'path' (value) não pode ser vazio"));
        }
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        println!("🟢 [FS-Browser] Processando requisição para: {}", input.value);

        // Usa input.value como o caminho a ser listado
        let path_clone = input.value.clone();

        let entries = tokio::task::spawn_blocking(move || {
            domain::get_entries(&path_clone)
        })
        .await
        .map_err(|_| AppError::Internal)? // Erro se a thread panicar
        ?; // Erro se o `get_entries` retornar um AppError

        println!("🟢 [FS-Browser] Enviando resposta com {} entradas.", entries.len());

        Ok(Output {
            // Retorna o input.value como current_path
            current_path: input.value,
            entries,
        })
    }
}

// Boilerplate CLI (sem alterações, apenas para manter a estrutura)
#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Configuração do CLI e Config
    let cli_cmd = <Cli as clap::CommandFactory>::command()
        .name("node-fs-browser")
        .about("Node de serviço para navegação no sistema de arquivos")
        .version(env!("CARGO_PKG_VERSION"));

    let matches = cli_cmd.get_matches();
    let args = Cli::from_arg_matches(&matches)
        .map_err(|e| AppError::bad(format!("erro ao parsear argumentos: {}", e)))?;

    let (mut cfg, cfg_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;
    println!("usando config: {}", cfg_path.display());

    if let Some(p) = args.port { cfg.port = p; }
    if cfg.port == 0 { return Err(AppError::bad(format!("Porta inválida ou não definida no config: {}", cfg_path.display()))); }

    // 2. Criação do Router, INJETANDO CORS
    let node = FsBrowserNode::default();
    let app_router = router(node);
    let cors = CorsLayer::permissive();
    let app = app_router.layer(cors);

    // 3. Servir a Aplicação
    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.port).parse().unwrap();
    println!("node-fs-browser ouvindo na porta {}", cfg.port);
    println!("listening on http://{addr}");

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.map_err(|_| AppError::Internal)
}