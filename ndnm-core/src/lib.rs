mod server;
mod error;
mod config;
mod runner;

pub use async_trait::async_trait;
pub use config::{NodeConfig, InputFieldConfig};
pub use runner::load_config;
pub use error::AppError;
pub use server::{router, serve, ServerOpts};
pub use runner::run_node;

use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait Node: Send + Sync + 'static {
    type Input: DeserializeOwned + Send;
    type Output: Serialize;

    fn validate(&self, _input: &Self::Input) -> Result<(), AppError> { Ok(()) }
    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError>;
}