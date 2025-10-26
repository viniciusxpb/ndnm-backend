// ndnm-core/src/config/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputFieldConfig {
    pub name: String,
    pub r#type: String,
}

// --- FORMATO LEGADO (compatibilidade com nodes antigos) ---
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NodeConfig {
    // Port é opcional no novo formato (Hermes gerencia portas)
    // Obrigatório no formato legado
    #[serde(default)]
    pub port: u16,
    pub label: Option<String>,
    pub node_type: Option<String>,
    pub inputs_mode: Option<String>,
    pub initial_inputs_count: Option<u16>,
    pub outputs_mode: Option<String>,
    pub initial_outputs_count: Option<u16>,
    #[serde(default)]
    pub input_fields: Vec<InputFieldConfig>,

    // --- NOVO FORMATO (para nodes avançados) ---
    // Se presente, este node usa o sistema de sections
    #[serde(default)]
    pub sections: Vec<Section>,
    pub node_id_hash: Option<String>,
}

// --- NOVO SISTEMA DE SECTIONS ---

/// Comportamento de uma seção de I/O
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SectionBehavior {
    /// Slots fixos e nomeados
    Fixed,
    /// UI adiciona slots dinamicamente quando conectados
    AutoIncrement,
    /// Slots criados dinamicamente baseados em arquivos no diretório
    DynamicPerFile,
}

/// Definição de um slot (input ou output)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotDefinition {
    /// Nome técnico do slot (usado como base para handleId)
    pub name: String,

    /// Label exibido na UI (pode conter placeholders como {filename})
    pub label: Option<String>,

    /// Tipo de dado (FILE_CONTENT, INT, STRING, etc.)
    pub r#type: String,

    /// Número de conexões permitidas (1 ou "n")
    #[serde(deserialize_with = "deserialize_connections")]
    pub connections: ConnectionMode,
}

/// Modo de conexão para um slot
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ConnectionMode {
    /// Exatamente uma conexão
    Single,
    /// Zero ou múltiplas conexões
    Multiple,
}

// Deserializer customizado para aceitar 1 ou "n"
fn deserialize_connections<'de, D>(deserializer: D) -> Result<ConnectionMode, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ConnectionValue {
        Number(u32),
        String(String),
    }

    match ConnectionValue::deserialize(deserializer)? {
        ConnectionValue::Number(1) => Ok(ConnectionMode::Single),
        ConnectionValue::Number(n) => Err(D::Error::custom(format!("Invalid connection count: {}. Use 1 or \"n\"", n))),
        ConnectionValue::String(s) if s == "n" => Ok(ConnectionMode::Multiple),
        ConnectionValue::String(s) => Err(D::Error::custom(format!("Invalid connection mode: {}. Use 1 or \"n\"", s))),
    }
}

/// Template de slot (par input/output)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotTemplate {
    /// Definição do input (se houver)
    pub input: Option<SlotDefinition>,

    /// Definição do output (se houver)
    pub output: Option<SlotDefinition>,
}

/// Uma seção de I/O no node
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Section {
    /// ID interno da seção
    pub section_name: String,

    /// Label opcional para agrupamento visual na UI
    pub section_label: Option<String>,

    /// Comportamento da seção
    pub behavior: SectionBehavior,

    /// Template de slots (para behaviors dinâmicos)
    pub slot_template: Option<SlotTemplate>,

    /// Slots fixos (para behavior: fixed com múltiplos slots)
    #[serde(default)]
    pub slots: Vec<SlotTemplate>,
}