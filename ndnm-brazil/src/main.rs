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
use ndnm_core::{AppError, load_config};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use chrono::Utc;

// --- NOSSA NOVA BRUXARIA: Estruturas para Configuração Dinâmica ---
use serde::Serialize;
use serde_json::json; // Para criar o `default_data` facilmente

/// Informação sobre um tipo de node disponível, enviada ao frontend.
#[derive(Serialize, Debug, Clone)]
struct NodeTypeInfo {
    r#type: String, // Usamos `r#` porque `type` é palavra reservada
    label: String,
    // Usamos `serde_json::Value` para flexibilidade nos dados default
    default_data: serde_json::Value,
}

/// Mensagem enviada pelo WebSocket do Brazil para o Frontend
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")] // Adiciona um campo "type" ao JSON final
enum BrazilToFrontend {
    #[serde(rename = "NODE_CONFIG")] // Nome do tipo no JSON
    NodeConfig { payload: Vec<NodeTypeInfo> },
    #[serde(rename = "ECHO")]
    Echo { message: String },
    // Adicionar outros tipos de mensagem aqui no futuro (ex: EXECUTION_RESULT)
}
// --- FIM DA BRUXARIA ---


// --- Configuração ---
#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[arg(short, long)]
    port: Option<u16>,
}

// --- Estado Compartilhado (Adicionamos a lista de nodes) ---
#[derive(Debug)]
struct AppState {
    tx: broadcast::Sender<String>,
    // Lista dos nodes que o Brazil conhece (será dinâmica no futuro)
    known_nodes: Vec<NodeTypeInfo>,
}

// --- Lógica Principal ---

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Cli::parse();
    let (mut config, config_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;

    println!(
        "{} | 🟢 [WS Brazil] ndnm-brazil (Maestro) usando config: {}",
        Utc::now().to_rfc3339(),
        config_path.display()
    );

    if let Some(p) = args.port {
        config.port = p;
    }

    // --- CRIA A LISTA DE NODES (HARDCODED POR AGORA) ---
    let known_nodes = vec![
        NodeTypeInfo {
            r#type: "textUpdater".to_string(),
            label: "📝 Texto".to_string(),
            default_data: json!({ "label": "Novo texto" }),
        },
        NodeTypeInfo {
            r#type: "add".to_string(),
            label: "➕ Somar".to_string(),
            default_data: json!({
                "label": "➕ Somar",
                "inputsMode": "n",
                "inputsCount": 1,
                "outputsMode": 1,
                "outputsCount": 1
            }),
        },
        NodeTypeInfo {
            r#type: "subtract".to_string(),
            label: "➖ Subtrair".to_string(),
            default_data: json!({
                "label": "➖ Subtrair",
                "inputsMode": "n",
                "inputsCount": 1,
                "outputsMode": 1,
                "outputsCount": 1
            }),
        },
        // TODO: Adicionar outros nodes conhecidos aqui (list-directory, etc.)
        // Futuramente, isso virá de um config ou service discovery
    ];
    println!("{} | 🟢 [WS Brazil] Nodes conhecidos: {:?}", Utc::now().to_rfc3339(), known_nodes.iter().map(|n| &n.r#type).collect::<Vec<_>>());


    // --- CRIA O ESTADO COMPARTILHADO ---
    let (tx, _) = broadcast::channel(100);
    // Passa a lista de nodes para o estado
    let app_state = Arc::new(AppState { tx, known_nodes });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state); // O estado agora inclui a lista de nodes

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
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
    State(state): State<Arc<AppState>>, // O estado agora tem known_nodes
) -> impl IntoResponse {
    println!("{} | 🟡 [WS Brazil] Novo cliente WebSocket tentando conectar...", Utc::now().to_rfc3339());
    ws.on_upgrade(|socket| handle_socket(socket, state)) // Passa o estado completo
}

// Função que gerencia uma conexão WebSocket individual
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("{} | 🟢 [WS Brazil] Cliente WebSocket CONECTADO!", Utc::now().to_rfc3339());
    let (mut sender, mut receiver) = socket.split();

    // --- ENVIA A CONFIGURAÇÃO INICIAL DE NODES ---
    let config_msg = BrazilToFrontend::NodeConfig { payload: state.known_nodes.clone() };
    match serde_json::to_string(&config_msg) {
        Ok(json_str) => {
            if sender.send(Message::Text(json_str)).await.is_err() {
                 println!("{} | 🔴 [WS Brazil] Falha ao enviar NODE_CONFIG inicial. Cliente desconectou cedo?", Utc::now().to_rfc3339());
                 return; // Aborta se não conseguir enviar a config inicial
            }
             println!("{} | 🟢 [WS Brazil] Enviou NODE_CONFIG inicial.", Utc::now().to_rfc3339());
        }
        Err(e) => {
             println!("{} | 🔴 [WS Brazil] Erro ao serializar NODE_CONFIG: {}", Utc::now().to_rfc3339(), e);
             return; // Aborta se houver erro de serialização
        }
    }
    // --- FIM DO ENVIO INICIAL ---


    let mut rx = state.tx.subscribe();

    // Loop de envio (broadcast + echo por enquanto)
    let state_clone_send = Arc::clone(&state); // Clone pro loop de envio
    let mut send_task = tokio::spawn(async move {
         // Primeiro, envia mensagens do broadcast (outros clientes)
        while let Ok(msg_from_broadcast) = rx.recv().await {
            // Verifica se a mensagem não é a própria config que acabamos de enviar (evita eco inicial)
            // Esta lógica pode precisar ser mais robusta se outras msgs puderem vir do broadcast
             if !msg_from_broadcast.contains("NODE_CONFIG") {
                if sender.send(Message::Text(msg_from_broadcast)).await.is_err() {
                    println!("{} | 🔴 [WS Brazil] Falha ao enviar msg BROADCAST. Cliente desconectou.", Utc::now().to_rfc3339());
                    break;
                }
            }
        }
    });


    // Loop de recebimento (processa msgs do cliente)
    let state_clone_recv = Arc::clone(&state); // Clone pro loop de recebimento
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("{} | 🟢 [WS Brazil] Recebido do cliente: {}", Utc::now().to_rfc3339(), text);

                    // TODO: Aqui virá a lógica de parsear o grafo JSON do frontend
                    // e chamar os nodes HTTP correspondentes.

                    // Por enquanto, só manda um ECHO de volta via broadcast
                    let echo_msg = BrazilToFrontend::Echo { message: format!("Brazil recebeu: {}", text) };
                    match serde_json::to_string(&echo_msg) {
                        Ok(json_str) => {
                             println!("{} | 🟢 [WS Brazil] Enviando ECHO via broadcast: {}", Utc::now().to_rfc3339(), json_str);
                             // Envia para TODOS os clientes conectados (incluindo o remetente)
                            if state_clone_recv.tx.send(json_str).is_err() {
                                 // Isso só falha se não houver NENHUM subscriber, o que é estranho aqui.
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
                _ => {} // Ignora outros tipos de mensagem (Binary, Ping, Pong...)
            }
        }
    });

    // Gerencia o ciclo de vida das tasks
    tokio::select! {
        res = (&mut send_task) => {
            println!("{} | 🟡 [WS Brazil] Task de ENVIO finalizada.", Utc::now().to_rfc3339());
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de envio: {:?}", Utc::now().to_rfc3339(), e); }
            recv_task.abort(); // Se envio morrer, mata recebimento
        },
        res = (&mut recv_task) => {
            println!("{} | 🟡 [WS Brazil] Task de RECEBIMENTO finalizada.", Utc::now().to_rfc3339());
            if let Err(e) = res { println!("{} | 🔴 [WS Brazil] Erro na task de recebimento: {:?}", Utc::now().to_rfc3339(), e); }
            send_task.abort(); // Se recebimento morrer, mata envio
        },
    };

    println!("{} | 🟡 [WS Brazil] Conexão WebSocket finalizada.", Utc::now().to_rfc3339());
}