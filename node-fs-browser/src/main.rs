// viniciusxpb/ndnm-backend/ndnm-backend-c893a1ebc17c6070ecb4b86d83dbca22839369a/node-fs-browser/src/main.rs
mod domain; 

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};
use domain::DirectoryEntry;

// NOVOS IMPORTS PARA SERVIDOR MANUAL E CORS
use ndnm_core::{router, load_config};
use tower_http::cors::CorsLayer;
use clap::{FromArgMatches, Parser};
use std::net::SocketAddr;
// IMPORT NECESSÁRIO PARA axum::serve
use axum; 
// IMPORT NECESSÁRIO PARA TcpListener (tokio)
use tokio::net::TcpListener;


// --- Estruturas de Comunicação (Input/Output) ---

#[derive(Debug, Deserialize)]
pub struct Input {
    path: String,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub current_path: String,
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
        if input.path.is_empty() {
            return Err(AppError::bad("O campo 'path' não pode ser vazio"));
        }
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        println!("🟢 [FS-Browser] Processando requisição para: {}", input.path);
        
        let path_clone = input.path.clone();

        let entries = tokio::task::spawn_blocking(move || {
            domain::get_entries(&path_clone)
        })
        .await
        .map_err(|_| AppError::Internal)?
        ?;

        println!("🟢 [FS-Browser] Enviando resposta com {} entradas.", entries.len());

        Ok(Output {
            current_path: input.path,
            entries,
        })
    }
}

// Boilerplate CLI (Copiado de ndnm-core::runner/mod.rs)
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
    let app_router = router(node); // Usa ndnm_core::router para criar a estrutura /health e /run

    // CORREÇÃO CORS: Aplica a camada CORS permissiva ao router
    let cors = CorsLayer::permissive();
    let app = app_router.layer(cors);

    // 3. Servir a Aplicação (Corrigido os imports)
    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.port).parse().unwrap();
    println!("node-fs-browser ouvindo na porta {}", cfg.port);
    println!("listening on http://{addr}");
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.map_err(|_| AppError::Internal)
}