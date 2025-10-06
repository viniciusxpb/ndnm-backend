// node-load-checkpoint/src/main.rs
mod domain;

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Input {
    file_path: String,
}

// Adicionamos um Enum para representar os tipos de modelo
#[derive(Debug, Serialize, Default, PartialEq)]
pub enum ModelType {
    #[default]
    Unknown,
    Checkpoint,
    Lora,
    TextualInversion,
}

#[derive(Debug, Serialize, Default)]
pub struct Output {
    file_path: String,
    model_type: ModelType,
    tensor_count: usize,
    // Estes campos agora são opcionais, pois só se aplicam a Checkpoints
    #[serde(skip_serializing_if = "Vec::is_empty")]
    model_keys_preview: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clip_keys_preview: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    vae_keys_preview: Vec<String>,
    #[serde(skip_serializing_if = "is_zero")]
    model_tensor_count: usize,
    #[serde(skip_serializing_if = "is_zero")]
    clip_tensor_count: usize,
    #[serde(skip_serializing_if = "is_zero")]
    vae_tensor_count: usize,
}

// Função auxiliar para o serde
fn is_zero(num: &usize) -> bool {
    *num == 0
}

#[derive(Default)]
pub struct LoadCheckpointNode;

#[async_trait]
impl Node for LoadCheckpointNode {
    type Input = Input;
    type Output = Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        let path_buf = Path::new(&input.file_path).to_path_buf();
        tokio::task::spawn_blocking(move || {
            domain::load_and_analyze_checkpoint(&path_buf)
        }).await.map_err(|_| AppError::Internal)?
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        LoadCheckpointNode::default(),
        "node-load-checkpoint",
        "Node que carrega e analisa um checkpoint de Stable Diffusion (.safetensors)",
        env!("CARGO_MANIFEST_DIR"),
    ).await
}