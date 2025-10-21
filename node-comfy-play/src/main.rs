// node-comfy-play/src/main.rs
use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};

/// Input para o Comfy Play
/// Fase 1: Só recebe comando "execute"
#[derive(Debug, Deserialize)]
pub struct Input {
    /// Ação a ser executada ("execute")
    action: String,
}

/// Output do Comfy Play
/// Fase 1: Só confirma execução
/// Nota: Este node NÃO tem saída no grafo (é terminal)
#[derive(Debug, Serialize)]
pub struct Output {
    /// Status da execução
    status: String,
    /// Mensagem descritiva
    message: String,
    /// Total de nodes executados (fase 2+)
    #[serde(skip_serializing_if = "Option::is_none")]
    nodes_executed: Option<usize>,
}

/// Node Comfy Play - O "Easy Mode" 🎮
///
/// Este é o Play simplificado inspirado no ComfyUI:
/// - Tem input (conecta em nodes anteriores)
/// - SEM output (nó terminal - fim da linha)
/// - Executa TUDO conectado a ele (sem granularidade)
/// - Perfeito pra usuários que querem simplicidade
///
/// Nome "Comfy" é uma homenagem sarcástica ao ComfyUI 😏
#[derive(Default)]
pub struct ComfyPlayNode;

#[async_trait]
impl Node for ComfyPlayNode {
    type Input = Input;
    type Output = Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        // Fase 1: Implementação simples
        // Só confirma que recebeu o comando
        //
        // Fases futuras (2+):
        // - Resolve dependências (TODOS os nodes conectados)
        // - Executa sequencialmente
        // - Salva cache
        // - Como é terminal (sem output), não dispara outros Plays

        if input.action == "execute" {
            Ok(Output {
                status: "started".to_string(),
                message: "ComfyUI-style execution started (Fase 1 - confirmação apenas)".to_string(),
                nodes_executed: None, // Fase 2: contar nodes executados
            })
        } else {
            Err(AppError::BadRequest(format!(
                "Invalid action: '{}'. Expected 'execute'",
                input.action
            )))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    ndnm_core::run_node(
        ComfyPlayNode::default(),
        "node-comfy-play",
        "ComfyUI-style Play node (terminal - no outputs) for simplified workflows",
        env!("CARGO_MANIFEST_DIR"),
    )
    .await
}
