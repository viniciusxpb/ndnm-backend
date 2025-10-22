// ndnm-brazil/src/execution/types.rs
//
// Tipos para o sistema de execução

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Requisição de execução vinda do frontend via WebSocket
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionRequest {
    /// ID do node Play que foi disparado
    pub play_node_id: String,
    /// ID do workspace atual (será usado em fases futuras para logging/caching)
    #[allow(dead_code)]
    pub workspace_id: String,
    /// Grafo completo (nodes + conexões)
    pub graph: WorkflowGraph,
}

/// Representação do grafo de workflow
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowGraph {
    /// Lista de todos os nodes no grafo
    pub nodes: Vec<GraphNode>,
    /// Lista de todas as conexões (edges)
    pub connections: Vec<Connection>,
}

/// Node no grafo
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphNode {
    /// ID único do node no grafo (ex: "node-sum-1")
    pub id: String,
    /// Tipo do node (ex: "add", "multiply", "playButton")
    pub node_type: String,
    /// Porta HTTP do node (ex: 3000)
    pub port: u16,
    /// Label para exibição (ex: "➕ Somar")
    pub label: String,
    /// Dados específicos do node (inputs do usuário)
    pub data: HashMap<String, serde_json::Value>,
}

/// Conexão entre dois nodes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Connection {
    /// ID do node de origem
    pub from_node_id: String,
    /// Index da saída no node de origem (0, 1, 2...)
    pub from_output_index: usize,
    /// ID do node de destino
    pub to_node_id: String,
    /// Index da entrada no node de destino (0, 1, 2...)
    pub to_input_index: usize,
}

/// Status de execução de um node individual
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum NodeExecutionStatus {
    Pending,
    Executing,
    Completed,
    Cached,
    Failed,
}

/// Resultado de execução de um node
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NodeExecutionResult {
    pub node_id: String,
    pub status: NodeExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub cached: bool,
}

/// Status geral da execução (enviado via WebSocket pro frontend)
/// Será usado na Fase 3 para updates em tempo real
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ExecutionStatus {
    pub run_id: String,
    pub status: ExecutionState,
    pub current_node: Option<String>,
    pub completed_nodes: Vec<String>,
    pub remaining_nodes: Vec<String>,
    pub total_nodes: usize,
    pub cached_nodes: usize,
}

/// Estado geral da execução
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ExecutionState {
    Starting,
    Executing,
    Completed,
    Failed,
}

/// Resultado final da execução
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub run_id: String,
    pub status: ExecutionState,
    pub total_nodes: usize,
    pub executed_nodes: usize,
    pub cached_nodes: usize,
    pub duration_ms: u64,
    pub error: Option<String>,
}
