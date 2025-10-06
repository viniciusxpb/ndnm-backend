// node-empty-latent-image/src/main.rs
mod domain;

use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Input {
    width: usize,
    height: usize,
    batch_size: usize,
}

#[derive(Debug, Serialize)]
pub struct Output {
    status: String,
    width: usize,
    height: usize,
    batch_size: usize,
    latent_width: usize,
    latent_height: usize,
    tensor_size: usize,
    data_type: String,
}

#[derive(Default)]
pub struct EmptyLatentImageNode;

#[async_trait]
impl Node for EmptyLatentImageNode {
    type Input = Input;
    type Output = Output;

    fn validate(&self, input: &Self::Input) -> Result<(), AppError> {
        if input.width % 8 != 0 || input.height % 8 != 0 {
            return Err(AppError::bad("Width and height must be divisible by 8"));
        }
        if input.batch_size == 0 {
            return Err(AppError::bad("Batch size must be greater than 0"));
        }
        if input.width > 16384 || input.height > 16384 {
            return Err(AppError::bad("Dimensions too large. Maximum: 16384x16384"));
        }
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        domain::create_empty_latent(&input)
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        EmptyLatentImageNode::default(),
        "node-empty-latent-image",
        "Node that creates a blank 'canvas' (latent image) for KSampler.",
        env!("CARGO_MANIFEST_DIR"),
    ).await
}