// ndnm-core/src/node/mod.rs
use crate::error::AppError;
use serde::{de::DeserializeOwned, Serialize};

pub use async_trait::async_trait;

#[async_trait]
pub trait Node: Send + Sync + 'static {
    type Input: DeserializeOwned + Send;
    type Output: Serialize;

    fn validate(&self, _input: &Self::Input) -> Result<(), AppError> {
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError>;
}