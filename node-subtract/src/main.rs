// node-subtract/src/main.rs
mod domain;

use ndnm_core::{AppError, Node};
use serde::{Deserialize, Serialize};

/// DTOs: Estruturas de dados para entrada e saída.
#[derive(Debug, Deserialize)]
pub struct Input { // Adicionado 'pub'
    variables: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct Output { // Adicionado 'pub'
    response: i64,
}

/// Implementação do Node para "subtract"
#[derive(Default)]
pub struct SubtractNode; // Adicionado 'pub'

impl Node for SubtractNode {
    type Input = Input;
    type Output = Output;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        if input.variables.len() < 2 {
            return Err(AppError::bad("envie ao menos 2 números em 'variables' para subtrair"));
        }
        if input.variables.len() > 1_000_000 {
            return Err(AppError::bad("máximo de 1e6 números"));
        }
        Ok(())
    }

    fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        let total = domain::subtract_all(&input.variables);
        Ok(Output { response: total })
    }
}

/// Ponto de entrada do programa.
#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        SubtractNode::default(),
        "node-subtract",
        "Node que subtrai uma lista de inteiros a partir do primeiro número",
        env!("CARGO_MANIFEST_DIR"),
    ).await
}