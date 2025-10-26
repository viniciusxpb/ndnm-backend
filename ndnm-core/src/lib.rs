// ndnm-core/src/lib.rs
pub mod node;
pub mod error;
pub mod config;
pub mod server;
pub mod runner;

// Exports públicos
pub use node::{Node, async_trait};
pub use error::AppError;
pub use config::{
    NodeConfig,
    InputFieldConfig,
    Section,
    SectionBehavior,
    SlotDefinition,
    SlotTemplate,
    ConnectionMode
};
pub use server::{router, serve, ServerOpts};
pub use runner::{run_node, load_config};