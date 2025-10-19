// ndnm-core/src/config.rs
use serde::Deserialize;

/// Representa a configuração completa de um Node, lida do config.yaml
#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    /// Porta onde o node HTTP irá rodar (obrigatório)
    pub port: u16,

    /// Nome amigável para exibir no frontend (opcional)
    pub label: Option<String>,

    /// O 'type' que o frontend usa para identificar o componente React (opcional)
    /// Se omitido, o Brazil tentará inferir do nome da pasta (ex: node-sum -> "sum")
    pub node_type: Option<String>,

    /// Modo de entrada: "0", "1", ou "n" (opcional, default "1")
    pub inputs_mode: Option<String>, // Usamos String pra facilitar o parse do YAML

    /// Contagem inicial de inputs (para mode="n") (opcional, default 1)
    pub initial_inputs_count: Option<u16>,

    /// Modo de saída: "0", "1", ou "n" (opcional, default "1")
    pub outputs_mode: Option<String>, // Usamos String pra facilitar o parse do YAML

     /// Contagem inicial de outputs (para mode="n") (opcional, default 1)
    pub initial_outputs_count: Option<u16>,

    // Podemos adicionar mais campos aqui no futuro (descrição, categoria, etc.)
}

// Implementação de `Default` para facilitar
// NOTA: O Default aqui NÃO define a porta. A porta é obrigatória no YAML.
//       Isso é mais um fallback para campos opcionais não presentes.
impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            port: 0, // Porta 0 indica que não foi carregada corretamente
            label: None,
            node_type: None,
            inputs_mode: Some("1".to_string()),
            initial_inputs_count: Some(1),
            outputs_mode: Some("1".to_string()),
            initial_outputs_count: Some(1),
        }
    }
}