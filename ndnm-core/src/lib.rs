// ndnm-core/src/lib.rs
//! Core: servidor genérico, erros, config e o trait Node.

mod server;
mod error;
mod config;
mod runner;

pub use async_trait::async_trait;
// --- MUDANÇA AQUI ---
pub use config::Config; // <- Já tinha
pub use runner::load_config; // <--- Adiciona essa linha
// --- FIM DA MUDANÇA ---
pub use error::AppError;
pub use server::{router, serve, ServerOpts};
pub use runner::run_node; // <- Já tinha

use serde::{de::DeserializeOwned, Serialize};

/// Trait que cada node implementa: define Input, Output e o `process`.
#[async_trait]
pub trait Node: Send + Sync + 'static {
    type Input: DeserializeOwned + Send;
    type Output: Serialize;

    /// (Opcional) validação leve antes de processar.
    fn validate(&self, _input: &Self::Input) -> Result<(), AppError> { Ok(()) }

    /// Regra de negócio do node. Agora é assíncrona.
    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError>;
}