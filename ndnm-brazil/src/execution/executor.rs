// ndnm-brazil/src/execution/executor.rs
//
// Executor sequencial de nodes
// Fase 2: Execução básica SEM cache (executa tudo sempre)

use super::types::*;
use super::resolver::DependencyResolver;
use reqwest::Client;
use std::time::Instant;
use chrono::Utc;

pub struct ExecutionEngine {
    /// Cliente HTTP para chamar os nodes
    http_client: Client,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// Executa um workflow a partir de um node Play
    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, String> {
        let start_time = Instant::now();

        // Gera run_id único
        let run_id = Self::generate_run_id();

        println!("🚀 Iniciando execução: run_id={}", run_id);
        println!("   Play node: {}", request.play_node_id);

        // Resolve dependências (grafo → lista ordenada)
        let resolver = DependencyResolver::new(&request.graph);
        let execution_order = resolver
            .resolve_from(&request.play_node_id)
            .map_err(|e| format!("Erro ao resolver dependências: {}", e))?;

        println!("   Ordem de execução: {:?}", execution_order.iter().map(|n| &n.id).collect::<Vec<_>>());
        println!("   Total de nodes: {}", execution_order.len());

        // Executa cada node sequencialmente
        let mut executed_count = 0;
        for node in &execution_order {
            // Skip do próprio Play node (ele não tem lógica de processamento)
            if node.node_type == "playButton" || node.node_type == "comfyPlay" {
                println!("⏭️  Pulando Play node: {}", node.id);
                continue;
            }

            println!("⚙️  Executando node: {} ({})", node.id, node.label);

            match self.execute_node(node).await {
                Ok(result) => {
                    println!("   ✅ Sucesso: {} em {}ms", node.id, result.duration_ms);
                    executed_count += 1;
                }
                Err(e) => {
                    println!("   ❌ Erro: {} - {}", node.id, e);
                    return Err(format!("Node {} falhou: {}", node.id, e));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        println!("🎉 Execução completa: run_id={}", run_id);
        println!("   Nodes executados: {}", executed_count);
        println!("   Duração total: {}ms", duration);

        Ok(ExecutionResult {
            run_id,
            status: ExecutionState::Completed,
            total_nodes: execution_order.len(),
            executed_nodes: executed_count,
            cached_nodes: 0, // Fase 2: sem cache ainda
            duration_ms: duration,
            error: None,
        })
    }

    /// Executa um node individual via HTTP POST
    async fn execute_node(&self, node: &GraphNode) -> Result<NodeExecutionResult, String> {
        let start_time = Instant::now();

        // Monta URL do node
        let url = format!("http://localhost:{}/run", node.port);

        // Fase 2: Por enquanto, envia os dados do node como input
        // Fase 3: Vai incluir outputs dos nodes anteriores
        let input_data = &node.data;

        // Faz POST HTTP pro node
        let response = self
            .http_client
            .post(&url)
            .json(input_data)
            .send()
            .await
            .map_err(|e| format!("Erro ao conectar com node {}: {}", node.id, e))?;

        let status_code = response.status();

        if !status_code.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Erro desconhecido".to_string());
            return Err(format!("Node retornou erro {}: {}", status_code, error_text));
        }

        // Parse da resposta JSON
        let output: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao parsear resposta do node {}: {}", node.id, e))?;

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Completed,
            output: Some(output),
            error: None,
            duration_ms: duration,
            cached: false,
        })
    }

    /// Gera run_id único (timestamp + random)
    fn generate_run_id() -> String {
        let now = Utc::now();
        format!("run_{}_{:x}", now.format("%Y-%m-%d_%H-%M-%S"), rand::random::<u32>())
    }
}

