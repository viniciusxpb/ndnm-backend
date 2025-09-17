mod domain;

use clap::Parser;
use ndnm_core::{AppError, Config, Node, ServerOpts};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// CLI: permite sobrescrever a porta do config.json.
#[derive(Parser, Debug)]
#[command(name = "node-sum", version, about = "Node que soma inteiros")]
struct Cli {
    /// Caminho do config.json (default: ./config.json)
    #[arg(long, default_value = "config.json")]
    config: String,

    /// Porta do servidor (sobrescreve a do config.json)
    #[arg(short, long)]
    port: Option<u16>,
}

/// DTOs
#[derive(Debug, Deserialize)]
struct Input {
    variables: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct Output {
    response: i64,
}

/// Implementação do Node para “sum”
struct SumNode;

impl Node for SumNode {
    type Input = Input;
    type Output = Output;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
    if input.variables.is_empty() {
        return Err(AppError::bad("envie ao menos 1 número em 'variables'"));
    }
    if input.variables.len() > 1_000_000 {
        return Err(AppError::bad("máximo de 1e6 números"));
    }
    Ok(())
}

    fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        let total = domain::sum_all(&input.variables);
        Ok(Output { response: total })
    }
}

fn try_read_config(path: &Path) -> Result<Config, AppError> {
    let data = fs::read_to_string(path).map_err(|e| {
        AppError::bad(format!("não consegui ler {:?}: {}", path, e))
    })?;
    serde_json::from_str::<Config>(&data).map_err(|e| {
        AppError::bad(format!("config inválido em {:?}: {}", path, e))
    })
}

/// Lê o config em duas tentativas:
/// 1) Caminho informado (relativo ao CWD)
/// 2) Mesmo caminho relativo ao diretório do crate (CARGO_MANIFEST_DIR)
fn load_config(cli_path: &str) -> Result<(Config, PathBuf), AppError> {
    let p1 = PathBuf::from(cli_path);
    if p1.exists() {
        let cfg = try_read_config(&p1)?;
        return Ok((cfg, p1));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p2 = manifest_dir.join(cli_path);
    if p2.exists() {
        let cfg = try_read_config(&p2)?;
        return Ok((cfg, p2));
    }

    Err(AppError::bad(format!(
        "não encontrei config.json em {:?} nem em {:?}",
        p1, p2
    )))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    // carrega config com fallback de diretório
    let (mut cfg, cfg_path) = load_config(&args.config)?;
    println!("usando config: {}", cfg_path.display());

    // CLI tem precedência
    if let Some(p) = args.port {
        cfg.port = p;
    }

    println!("node-sum ouvindo na porta {}", cfg.port);
    ndnm_core::serve(ServerOpts { port: cfg.port }, SumNode).await
}
