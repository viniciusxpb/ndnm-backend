// ndnm-brazil/src/main.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use futures_util::{sink::SinkExt, stream::StreamExt};
// --- MUDANÇA AQUI ---
use ndnm_core::{AppError, load_config, Config as CoreConfig}; // Reutilizamos AppError E load_config! E renomeamos Config para CoreConfig para evitar conflito
// --- FIM DA MUDANÇA ---
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;

// --- Configuração ---
#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "config.json")]
    config: String,
    #[arg(short, long)]
    port: Option<u16>,
}

// Não precisamos mais desta struct Config local
// #[derive(serde::Deserialize, Debug, Clone)]
// struct Config {
//     port: u16,
// }

// Estado compartilhado do servidor
#[derive(Debug)]
struct AppState {
    tx: broadcast::Sender<String>,
}

// --- Lógica Principal ---

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Parsear args e carregar config (AGORA USANDO ndnm_core::load_config)
    let args = Cli::parse();
    // --- MUDANÇA AQUI ---
    // Passamos o caminho do config e o diretório do manifesto do crate atual
    let (mut config, config_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;
    // Usamos CoreConfig que importamos do ndnm_core
    // --- FIM DA MUDANÇA ---

    println!(
        "ndnm-brazil (Maestro) usando config: {}",
        config_path.display()
    );

    // Sobrescreve a porta se vier pelo CLI
    if let Some(p) = args.port {
        config.port = p;
    }


    // 2. Criar estado compartilhado
    let (tx, _) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx });

    // 3. Definir as rotas (continua igual)
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    // 4. Iniciar o servidor (continua igual)
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("ndnm-brazil ouvindo em {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(())
}

// --- Handlers (ws_handler, handle_socket, health_handler continuam iguais) ---

// Handler HTTP simples pra saber se está vivo
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Brazil is alive!")
}

// Handler que lida com a conexão WebSocket
async fn ws_handler(
    ws: WebSocketUpgrade,            // Ferramenta do Axum pra fazer o upgrade da conexão
    State(state): State<Arc<AppState>>, // Acesso ao nosso estado compartilhado
) -> impl IntoResponse {
    println!("Novo cliente WebSocket conectando...");
    // Finaliza o handshake e inicia a comunicação WebSocket
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// Função que gerencia uma conexão WebSocket individual
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("Cliente WebSocket conectado!");
    let (mut sender, mut receiver) = socket.split();

    // Clona o transmissor do broadcast para este cliente
    let mut rx = state.tx.subscribe();

    // --- Loop para Enviar Mensagens do Servidor para o Cliente ---
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Se não conseguir enviar (cliente desconectou?), para a tarefa
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // --- Loop para Receber Mensagens do Cliente para o Servidor ---
    let tx = state.tx.clone(); // Clona o transmissor para usar neste loop
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("Recebido do cliente: {}", text);
                    // --- AQUI VAI A LÓGICA PRINCIPAL ---
                    // TODO: Implementar essa lógica!

                    // Por enquanto, só manda uma mensagem de volta pra confirmar
                    let response = format!("Brazil recebeu: {}", text);
                    // Manda a resposta para TODOS os clientes conectados
                    if tx.send(response).is_err() {
                        // Se ninguém está ouvindo, não faz mal
                    }
                }
                Message::Close(_) => {
                    println!("Cliente desconectou.");
                    break;
                }
                _ => { /* Ignora outros tipos de mensagem (Binary, Ping, Pong) */ }
            }
        }
    });

    // Mantém a conexão aberta até uma das tarefas (envio ou recebimento) terminar
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("Conexão WebSocket finalizada.");
}