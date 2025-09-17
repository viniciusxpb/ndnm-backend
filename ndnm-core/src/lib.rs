//! Core: servidor genérico, erros, config e o trait Node.

mod server;
mod error;
mod config;

pub use config::Config;
pub use error::AppError;
pub use server::{router, serve, ServerOpts};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Trait que cada node implementa: define Input, Output e o `process`.
pub trait Node: Send + Sync + 'static {
    type Input: DeserializeOwned;
    type Output: Serialize;

    /// (Opcional) validação leve antes de processar.
    fn validate(&self, _input: &Self::Input) -> Result<(), AppError> { Ok(()) }

    /// Regra de negócio do node.
    fn process(&self, input: Self::Input) -> Result<Self::Output, AppError>;
}
