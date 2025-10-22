// ndnm-brazil/src/execution/mod.rs
//
// Motor de execução do ndnm - Fase 2
// Responsável por executar nodes sequencialmente seguindo o grafo de dependências

pub mod types;
pub mod resolver;
pub mod executor;

pub use types::*;
pub use executor::ExecutionEngine;
