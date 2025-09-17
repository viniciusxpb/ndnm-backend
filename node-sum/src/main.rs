mod domain;

use clap::Parser;
use ndnm_core::{AppError, Config, Node, ServerOpts};
use serde::{Deserialize, Serialize};
use std::fs;

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

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    let cfg_data = fs::read_to_string(&args.config)
        .expect("não consegui ler config.json");
    let mut cfg: Config = serde_json::from_str(&cfg_data)
        .expect("config.json inválido");

    if let Some(p) = args.port { cfg.port = p; }

    ndnm_core::serve(ServerOpts { port: cfg.port }, SumNode).await
}
