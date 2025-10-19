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
// Usamos AppError, load_config e renomeamos o Config do core
use ndnm_core::{AppError, load_config};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;

// --- NOSSA NOVA BRUXARIA DE LOG ---
use chrono::Utc;

// --- Configuração ---
#[derive(Parser, Debug)]
struct Cli {
    /// Caminho do config.yaml (default: ./config.yaml)
    #[arg(long, default_value = "config.yaml")]
    config: String,

    /// Porta do servidor (sobrescreve a do config.yaml)
    #[arg(short, long)]
    port: Option<u16>,
}

// Estado compartilhado do servidor
#[derive(Debug)]
struct AppState {
    tx: broadcast::Sender<String>,
}

// --- Lógica Principal ---

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Parsear args e carregar config
    let args = Cli::parse();
    let (mut config, config_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;

    println!(
        "{} | 🟢 [WS Brazil] ndnm-brazil (Maestro) usando config: {}",
        Utc::now().to_rfc3339(), // <--- LOG COM TIMESTAMP
        config_path.display()
    );

    // Sobrescreve a porta se vier pelo CLI
    if let Some(p) = args.port {
        config.port = p;
    }

    // 2. Criar estado compartilhado
    let (tx, _) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx });

    // 3. Definir as rotas
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    // 4. Iniciar o servidor
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("{} | 🟢 [WS Brazil] ndnm-brazil ouvindo em {}", Utc::now().to_rfc3339(), addr); // <--- LOG COM TIMESTAMP
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(())
}

// --- Handlers ---

// Handler HTTP simples pra saber se está vivo
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Brazil is alive!")
}

// Handler que lida com a conexão WebSocket
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("{} | 🟡 [WS Brazil] Novo cliente WebSocket tentando conectar...", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// Função que gerencia uma conexão WebSocket individual
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("{} | 🟢 [WS Brazil] Cliente WebSocket CONECTADO!", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    // --- Loop para Enviar Mensagens do Servidor para o Cliente ---
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                println!("{} | 🔴 [WS Brazil] Falha ao enviar msg para o cliente (loop de envio). Cliente provavelmente desconectou.", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
                break;
            }
        }
    });

    // --- Loop para Receber Mensagens do Cliente para o Servidor ---
    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("{} | 🟢 [WS Brazil] Recebido do cliente: {}", Utc::now().to_rfc3339(), text); // <--- LOG COM TIMESTAMP
                    // TODO: Implementar lógica de parse/execução do grafo

                    let response = format!("Brazil recebeu: {}", text);
                    println!("{} | 🟢 [WS Brazil] Enviando resposta: {}", Utc::now().to_rfc3339(), response); // <--- LOG COM TIMESTAMP
                    if tx.send(response).is_err() {
                        // Ninguém ouvindo
                    }
                }
                Message::Close(close_frame) => {
                    if let Some(frame) = close_frame {
                        println!("{} | 🟡 [WS Brazil] Cliente desconectou com frame: code={}, reason={}", Utc::now().to_rfc3339(), frame.code, frame.reason); // <--- LOG COM TIMESTAMP
                    } else {
                        println!("{} | 🟡 [WS Brazil] Cliente desconectou (sem frame).", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        res = (&mut send_task) => {
            println!("{} | 🟡 [WS Brazil] Task de ENVIO finalizada.", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de envio: {:?}", Utc::now().to_rfc3339(), e); } // <--- LOG COM TIMESTAMP
            recv_task.abort();
        },
        res = (&mut recv_task) => {
            println!("{} | 🟡 [WS Brazil] Task de RECEBIMENTO finalizada.", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de recebimento: {:?}", Utc::now().to_rfc3339(), e); } // <--- LOG COM TIMESTAMP
            send_task.abort();
        },
    };

    println!("{} | 🟡 [WS Brazil] Conexão WebSocket finalizada.", Utc::now().to_rfc3339()); // <--- LOG COM TIMESTAMP
}