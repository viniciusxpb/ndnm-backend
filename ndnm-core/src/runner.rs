// ndnm-core/src/runner.rs
use crate::{AppError, Config, Node, ServerOpts};
use clap::{FromArgMatches, Parser};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// CLI genérica para qualquer node que use o ndnm-core.
#[derive(Parser, Debug)]
struct Cli {
    /// Caminho do config.json (default: ./config.json)
    #[arg(long, default_value = "config.json")]
    config: String,

    /// Porta do servidor (sobrescreve a do config.json)
    #[arg(short, long)]
    port: Option<u16>,
}

/// A função principal que inicializa e serve um Node.
pub async fn run_node<N>(node: N, name: &'static str, about: &'static str, node_manifest_dir: &'static str) -> Result<(), AppError>
where
    N: Node + Send + Sync + 'static,
    N::Input: serde::de::DeserializeOwned + Send + 'static,
    N::Output: serde::Serialize + Send + 'static,
{
    let cli_cmd = <Cli as clap::CommandFactory>::command()
        .name(name)
        .about(about)
        .version(env!("CARGO_PKG_VERSION"));

    let matches = cli_cmd.get_matches();
    let args = Cli::from_arg_matches(&matches)
        .map_err(|e| AppError::bad(format!("erro ao parsear argumentos: {}", e)))?;

    let (mut cfg, cfg_path) = load_config(&args.config, node_manifest_dir)?; // <--- Chamada aqui
    println!("usando config: {}", cfg_path.display());

    if let Some(p) = args.port {
        cfg.port = p;
    }

    println!("{} ouvindo na porta {}", name, cfg.port);
    crate::server::serve(ServerOpts { port: cfg.port }, node).await
}

// Função interna que lê e parseia o config
fn try_read_config(path: &Path) -> Result<Config, AppError> {
    let data = fs::read_to_string(path)
        .map_err(|e| AppError::bad(format!("não consegui ler {:?}: {}", path, e)))?;
    serde_json::from_str::<Config>(&data)
        .map_err(|e| AppError::bad(format!("config inválido em {:?}: {}", path, e)))
}

// --- MUDANÇA AQUI ---
/// Carrega a configuração, procurando no path do CLI e como fallback no diretório do manifesto do node.
pub fn load_config(cli_path: &str, node_manifest_dir: &str) -> Result<(Config, PathBuf), AppError> { // <--- Adicionado `pub`
    let p1 = PathBuf::from(cli_path);
    if p1.exists() {
        let cfg = try_read_config(&p1)?;
        return Ok((cfg, p1));
    }
    
    let manifest_dir = PathBuf::from(node_manifest_dir);
    let p2 = manifest_dir.join(cli_path);
    if p2.exists() {
        let cfg = try_read_config(&p2)?;
        return Ok((cfg, p2));
    }

    Err(AppError::bad(format!(
        "não encontrei config.json em {:?} nem em {:?}",
        p1.display(), p2.display()
    )))
}