// node-load-checkpoint/src/main.rs
mod domain;

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Input {
    file_path: String,
}

#[derive(Debug, Serialize)]
pub struct Output {
    file_path: String,
    model_tensor_count: usize,
    clip_tensor_count: usize,
    vae_tensor_count: usize,
    model_keys_preview: Vec<String>,
    clip_keys_preview: Vec<String>,
    vae_keys_preview: Vec<String>,
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