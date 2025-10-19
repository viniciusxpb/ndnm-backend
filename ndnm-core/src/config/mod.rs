// ndnm-core/src/config/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputFieldConfig {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NodeConfig {
    pub port: u16,
    pub label: Option<String>,
    pub node_type: Option<String>,
    pub inputs_mode: Option<String>,
    pub initial_inputs_count: Option<u16>,
    pub outputs_mode: Option<String>,
    pub initial_outputs_count: Option<u16>,
    #[serde(default)]
    pub input_fields: Vec<InputFieldConfig>,
}