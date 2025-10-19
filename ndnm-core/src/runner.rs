// ndnm-core/src/runner.rs
// --- CORREÇÃO AQUI ---
// Importa NodeConfig diretamente do módulo config
use crate::{AppError, config::NodeConfig, Node, ServerOpts};
// --- FIM DA CORREÇÃO ---
use clap::{FromArgMatches, Parser};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// CLI genérica para qualquer node que use o ndnm-core.
#[derive(Parser, Debug)]
struct Cli {
    /// Caminho do config.yaml (default: ./config.yaml)
    #[arg(long, default_value = "config.yaml")]
    config: String,

    /// Porta do servidor (sobrescreve a do config.yaml)
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

    let (mut cfg, cfg_path) = load_config(&args.config, node_manifest_dir)?;
    println!("usando config: {}", cfg_path.display());

    if let Some(p) = args.port {
        cfg.port = p;
    }

    if cfg.port == 0 {
         return Err(AppError::bad(format!(
            "Porta inválida ou não definida no config: {}",
            cfg_path.display()
        )));
    }


    println!("{} ouvindo na porta {}", name, cfg.port);
    crate::server::serve(ServerOpts { port: cfg.port }, node).await
}

/// Tenta ler e parsear um arquivo de configuração YAML para NodeConfig.
fn try_read_config(path: &Path) -> Result<NodeConfig, AppError> {
    let data = fs::read_to_string(path)
        .map_err(|e| AppError::bad(format!("não consegui ler {:?}: {}", path, e)))?;
    serde_yaml::from_str::<NodeConfig>(&data)
        .map_err(|e| AppError::bad(format!("config inválido em {:?}: {}", path, e)))
}

/// Carrega a configuração (`NodeConfig`), procurando no path do CLI e como fallback no diretório do manifesto do node.
pub fn load_config(cli_path: &str, node_manifest_dir: &str) -> Result<(NodeConfig, PathBuf), AppError> {
    let p1 = PathBuf::from(cli_path);
    if p1.exists() {
        let cfg = try_read_config(&p1)?;
        return Ok((cfg, p1));
    }

    let manifest_dir = PathBuf::from(node_manifest_dir);
    let file_name = Path::new(cli_path).file_name().unwrap_or_else(|| Path::new("config.yaml").as_os_str());
    let p2 = manifest_dir.join(file_name);
    if p2.exists() {
        let cfg = try_read_config(&p2)?;
        return Ok((cfg, p2));
    }

    Err(AppError::bad(format!(
        "não encontrei {} em {:?} nem em {:?}",
        file_name.to_string_lossy(),
        p1.parent().unwrap_or_else(|| Path::new(".")).display(),
        p2.parent().unwrap_or_else(|| Path::new(".")).display()
    )))
}