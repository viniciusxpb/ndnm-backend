// node-sum/src/main.rs
mod domain;

use ndnm_core::{AppError, Node};
use serde::{Deserialize, Serialize};

/// DTOs: Estruturas de dados para entrada e saída.
#[derive(Debug, Deserialize)]
struct Input {
    variables: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct Output {
    response: i64,
}

/// Implementação do Node para "sum"
#[derive(Default)]
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

/// Ponto de entrada do programa.
#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        SumNode::default(),
        "node-sum",
        "Node que soma uma lista de inteiros",
        env!("CARGO_MANIFEST_DIR"), // Corrigido: Passa o caminho do seu próprio diretório.
    ).await
}