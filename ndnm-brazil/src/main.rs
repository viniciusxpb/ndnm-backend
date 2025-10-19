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
// --- CORREÇÃO AQUI ---
// Removemos `NodeConfig` daqui, pois `load_config` já a retorna.
use ndnm_core::{AppError, load_config};
// --- FIM DA CORREÇÃO ---
// --- CORREÇÃO AQUI ---
// Removemos `path::PathBuf` daqui, pois só é usado em `discover_nodes`
use std::{net::SocketAddr, sync::Arc};
// --- FIM DA CORREÇÃO ---
use tokio::sync::broadcast;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use walkdir::WalkDir; // Este use está correto e é necessário

// --- Estruturas de Comunicação ---
#[derive(Serialize, Debug, Clone)]
struct NodeTypeInfo {
    r#type: String,
    label: String,
    default_data: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
enum BrazilToFrontend {
    #[serde(rename = "NODE_CONFIG")]
    NodeConfig { payload: Vec<NodeTypeInfo> },
    #[serde(rename = "ECHO")]
    Echo { message: String },
}

// --- Configuração CLI ---
#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[arg(short, long)]
    port: Option<u16>,
}

// --- Estado Compartilhado ---
#[derive(Debug)]
struct AppState {
    tx: broadcast::Sender<String>,
    known_nodes: Vec<NodeTypeInfo>,
}

// --- Função Auxiliar: Descobrir Nodes ---
fn discover_nodes() -> Vec<NodeTypeInfo> {
    // Importamos PathBuf aqui dentro, onde é usado
    use std::path::{Path, PathBuf};

    let mut discovered_nodes = Vec::new();
    let current_dir = std::env::current_dir().expect("Não consegui ler o diretório atual");
    let workspace_dir = current_dir.parent().unwrap_or(&current_dir);

    println!("{} | 🟡 [WS Brazil] Procurando nodes em: {}", Utc::now().to_rfc3339(), workspace_dir.display());

    for entry in WalkDir::new(workspace_dir).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with("node-") {
                    println!("{} | 🟡 [WS Brazil] Encontrei potencial node: {}", Utc::now().to_rfc3339(), dir_name);
                    let config_path = path.join("config.yaml");
                     match load_config(config_path.to_str().unwrap_or(""), path.to_str().unwrap_or("")) {
                        Ok((node_config, _)) => { // node_config aqui é do tipo ndnm_core::NodeConfig
                            println!("{} | 🟢 [WS Brazil]  -> Config carregado para '{}'", Utc::now().to_rfc3339(), dir_name);

                            let node_type = node_config.node_type
                                .clone()
                                .unwrap_or_else(|| dir_name.trim_start_matches("node-").to_string());

                            let label = node_config.label.clone().unwrap_or_else(|| node_type.clone());

                            let default_data = json!({
                                "label": label,
                                "inputsMode": node_config.inputs_mode.unwrap_or_else(|| "1".to_string()),
                                "inputsCount": node_config.initial_inputs_count.unwrap_or(1),
                                "outputsMode": node_config.outputs_mode.unwrap_or_else(|| "1".to_string()),
                                "outputsCount": node_config.initial_outputs_count.unwrap_or(1),
                            });

                            discovered_nodes.push(NodeTypeInfo {
                                r#type: node_type,
                                label: label,
                                default_data,
                            });
                        }
                        Err(e) => {
                             println!("{} | 🔴 [WS Brazil]  -> Falha ao carregar config para '{}': {}", Utc::now().to_rfc3339(), dir_name, e);
                        }
                    }
                }
            }
        }
    }

    discovered_nodes.sort_by(|a, b| a.label.cmp(&b.label));
    discovered_nodes
}

// --- Lógica Principal ---
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Cli::parse();
    let (mut brazil_config, config_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;

    println!(
        "{} | 🟢 [WS Brazil] ndnm-brazil (Maestro) usando config: {}",
        Utc::now().to_rfc3339(),
        config_path.display()
    );

    if let Some(p) = args.port {
        brazil_config.port = p;
    }
     if brazil_config.port == 0 {
         return Err(AppError::bad(format!(
            "Porta inválida ou não definida no config do Brazil: {}",
            config_path.display()
        )));
    }

    let discovered_nodes = discover_nodes();
    println!("{} | 🟢 [WS Brazil] Nodes descobertos: {:?}", Utc::now().to_rfc3339(), discovered_nodes.iter().map(|n| &n.r#type).collect::<Vec<_>>());

    let (tx, _) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx, known_nodes: discovered_nodes });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], brazil_config.port));
    println!("{} | 🟢 [WS Brazil] ndnm-brazil ouvindo em {}", Utc::now().to_rfc3339(), addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(())
}

// --- Handlers ---
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Brazil is alive!")
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("{} | 🟡 [WS Brazil] Novo cliente WebSocket tentando conectar...", Utc::now().to_rfc3339());
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
     println!("{} | 🟢 [WS Brazil] Cliente WebSocket CONECTADO!", Utc::now().to_rfc3339());
    let (mut sender, mut receiver) = socket.split();

    let config_msg = BrazilToFrontend::NodeConfig { payload: state.known_nodes.clone() };
    match serde_json::to_string(&config_msg) {
        Ok(json_str) => {
            if sender.send(Message::Text(json_str)).await.is_err() {
                 println!("{} | 🔴 [WS Brazil] Falha ao enviar NODE_CONFIG inicial. Cliente desconectou cedo?", Utc::now().to_rfc3339());
                 return;
            }
             println!("{} | 🟢 [WS Brazil] Enviou NODE_CONFIG inicial.", Utc::now().to_rfc3339());
        }
        Err(e) => {
             println!("{} | 🔴 [WS Brazil] Erro ao serializar NODE_CONFIG: {}", Utc::now().to_rfc3339(), e);
             return;
        }
    }

    let mut rx = state.tx.subscribe();
    // Prefixamos com _ pois não usamos diretamente no loop de envio
    let _state_clone_send = Arc::clone(&state);
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg_from_broadcast) = rx.recv().await {
             if !msg_from_broadcast.contains("\"type\":\"NODE_CONFIG\"") {
                 if sender.send(Message::Text(msg_from_broadcast)).await.is_err() {
                    println!("{} | 🔴 [WS Brazil] Falha ao enviar msg BROADCAST. Cliente desconectou.", Utc::now().to_rfc3339());
                    break;
                }
             }
        }
    });

    let state_clone_recv = Arc::clone(&state);
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("{} | 🟢 [WS Brazil] Recebido do cliente: {}", Utc::now().to_rfc3339(), text);
                    let echo_msg = BrazilToFrontend::Echo { message: format!("Brazil recebeu: {}", text) };
                    match serde_json::to_string(&echo_msg) {
                        Ok(json_str) => {
                             println!("{} | 🟢 [WS Brazil] Enviando ECHO via broadcast: {}", Utc::now().to_rfc3339(), json_str);
                            if state_clone_recv.tx.send(json_str).is_err() {
                                 println!("{} | 🟡 [WS Brazil] Aviso: Nenhum cliente ouvindo o broadcast.", Utc::now().to_rfc3339());
                            }
                        }
                        Err(e) => {
                             println!("{} | 🔴 [WS Brazil] Erro ao serializar ECHO: {}", Utc::now().to_rfc3339(), e);
                        }
                    }
                }
                Message::Close(close_frame) => {
                    if let Some(frame) = close_frame {
                        println!("{} | 🟡 [WS Brazil] Cliente desconectou com frame: code={}, reason={}", Utc::now().to_rfc3339(), frame.code, frame.reason);
                    } else {
                        println!("{} | 🟡 [WS Brazil] Cliente desconectou (sem frame).", Utc::now().to_rfc3339());
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        res = (&mut send_task) => {
            println!("{} | 🟡 [WS Brazil] Task de ENVIO finalizada.", Utc::now().to_rfc3339());
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de envio: {:?}", Utc::now().to_rfc3339(), e); }
            recv_task.abort();
        },
        res = (&mut recv_task) => {
            println!("{} | 🟡 [WS Brazil] Task de RECEBIMENTO finalizada.", Utc::now().to_rfc3339());
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de recebimento: {:?}", Utc::now().to_rfc3339(), e); }
            send_task.abort();
        },
    };

    println!("{} | 🟡 [WS Brazil] Conexão WebSocket finalizada.", Utc::now().to_rfc3339());
}