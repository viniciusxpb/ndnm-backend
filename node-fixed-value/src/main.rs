// node-fixed-value/src/main.rs
// Por enquanto, este node só existe para definir sua estrutura e valor.
// Ele recebe um valor (que viria do input de texto no front) e o retorna.
// A lógica de passar o valor para as saídas será implementada depois.

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};

// O input esperado (eventualmente virá do estado do frontend)
#[derive(Debug, Deserialize, Clone)]
pub struct Input {
    value: String, // O valor fixo que o usuário digita
}

// O output que o node retorna (por enquanto, só ecoa o valor)
#[derive(Debug, Serialize)]
pub struct Output {
    response_value: String,
}

#[derive(Default)]
pub struct FixedValueNode;

#[async_trait]
impl Node for FixedValueNode {
    type Input = Input; // Na prática, o Brazil não enviará isso ainda
    type Output = Output;

    // Não precisa de validação complexa por enquanto
    fn validate(&self, _input: &Self::Input) -> Result<(), AppError> {
        Ok(())
    }

    // Apenas retorna o valor recebido (simulação)
    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        // No futuro, o `ndnm-brazil` pegará esse valor do estado do grafo,
        // não necessariamente de um POST /run direto para este node.
        Ok(Output { response_value: input.value })
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        FixedValueNode::default(),
        "node-fixed-value",
        "Node que armazena um valor fixo e o disponibiliza em saídas dinâmicas",
        env!("CARGO_MANIFEST_DIR"),
    ).await
}