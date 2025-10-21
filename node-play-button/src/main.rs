// node-play-button/src/main.rs
use ndnm_core::{async_trait, AppError, Node};
use serde::{Deserialize, Serialize};

/// Input para o Play Button
/// Fase 1: Só recebe comando "execute"
/// Fases futuras: Receberá grafo completo para executar
#[derive(Debug, Deserialize)]
pub struct Input {
    /// Ação a ser executada ("execute" por enquanto)
    action: String,
}

/// Output do Play Button
/// Fase 1: Só confirma que recebeu o comando
/// Fases futuras: Retornará resultado da execução completa
#[derive(Debug, Serialize)]
pub struct Output {
    /// Status da execução ("started", "completed", "error")
    status: String,
    /// Mensagem descritiva
    message: String,
    /// Run ID (será adicionado em fases futuras)
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
}

/// Node Play Button - O CHAD Play 💪
///
/// Este é o Play avançado que tem:
/// - Input (conecta em nodes anteriores)
/// - Output (pode disparar outro Play - cascata!)
/// - Cache inteligente (fase 2+)
/// - Resolução de dependências (fase 3+)
#[derive(Default)]
pub struct PlayButtonNode;

#[async_trait]
impl Node for PlayButtonNode {
    type Input = Input;
    type Output = Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        // Fase 1: Implementação SUPER simples
        // Só confirma que recebeu o comando "execute"
        //
        // Fases futuras (2+): Aqui vai:
        // 1. Resolver dependências (quais nodes executar)
        // 2. Calcular hashes
        // 3. Executar nodes sequencialmente
        // 4. Salvar cache
        // 5. Retornar resultado

        if input.action == "execute" {
            // Por enquanto: só confirma que recebeu
            // TODO (Fase 2): Disparar execução via ndnm-brazil
            Ok(Output {
                status: "started".to_string(),
                message: "Play execution started (Fase 1 - confirmação apenas)".to_string(),
                run_id: None, // Fase 2: gerar run_id aqui
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
        PlayButtonNode::default(),
        "node-play-button",
        "Advanced Play node with input/output for cascading executions",
        env!("CARGO_MANIFEST_DIR"),
    )
    .await
}
