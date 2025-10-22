// ndnm-brazil/src/main.rs

// Módulo de execução (Fase 2)
mod execution;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clap::Parser;
use futures_util::{stream::StreamExt, sink::SinkExt};
use ndnm_core::{AppError, load_config};
use std::{fs, net::SocketAddr, path::Path as StdPath, sync::Arc};
use tokio::sync::broadcast;
// FIX E0412: Importado DateTime
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use walkdir::WalkDir;
use tower_http::cors::CorsLayer;
use reqwest::Client; // Cliente HTTP para chamar o node-fs-browser

// --- Protocolo WS para o Frontend ---

#[derive(Serialize, Debug, Clone)]
struct NodeTypeInfo {
    r#type: String,
    label: String,
    default_data: Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
enum BrazilToFrontend {
    #[serde(rename = "NODE_CONFIG")]
    NodeConfig { payload: Vec<NodeTypeInfo> },
    #[serde(rename = "ECHO")]
    Echo { message: String },
    // NOVO: Resposta da navegação de arquivos
    #[serde(rename = "FS_BROWSE_RESULT")]
    FsBrowseResult {
        current_path: String,
        entries: Vec<DirectoryEntry>
    },
    // NOVO (Fase 2): Status de execução em tempo real
    // Será usado na Fase 3 para updates em tempo real
    #[allow(dead_code)]
    #[serde(rename = "EXECUTION_STATUS")]
    ExecutionStatus {
        run_id: String,
        status: String,
        current_node: Option<String>,
        completed_nodes: Vec<String>,
        remaining_nodes: Vec<String>,
    },
    // NOVO (Fase 2): Resultado final da execução
    #[serde(rename = "EXECUTION_COMPLETE")]
    ExecutionComplete {
        run_id: String,
        status: String,
        total_nodes: usize,
        executed_nodes: usize,
        cached_nodes: usize,
        duration_ms: u64,
    },
    // NOVO (Fase 2): Erro durante execução
    #[serde(rename = "EXECUTION_ERROR")]
    ExecutionError {
        run_id: String,
        error: String,
        failed_node: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum FrontendToBrazil {
    // FIX Warning: request_id é mantido para compatibilidade, mas ignorado no processamento
    #[serde(rename = "BROWSE_PATH")]
    BrowsePath {
        path: String,
        #[allow(dead_code)]
        request_id: String
    },
    #[serde(rename = "ECHO")]
    Echo { message: String },
    // NOVO (Fase 2): Executar Play node
    #[serde(rename = "EXECUTE_PLAY")]
    ExecutePlay {
        play_node_id: String,
        workspace_id: String,
        graph: execution::WorkflowGraph,
    },
}

// Estrutura do node-fs-browser
#[derive(Serialize, Deserialize, Debug)]
struct FsBrowserInput {
    path: String,
}

// Estrutura do Output do node-fs-browser
// FIX E0277: Adicionado Clone para que o vetor em BrazilToFrontend seja clonável
#[derive(Serialize, Deserialize, Debug, Clone)] 
struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub path: String,
    pub modified: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FsBrowserOutput {
    pub current_path: String,
    pub entries: Vec<DirectoryEntry>,
}


#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "config.yaml")]
    config: String,
    #[arg(short, long)]
    port: Option<u16>,
}

#[derive(Debug)]
struct AppState {
    tx: broadcast::Sender<String>,
    known_nodes: Vec<NodeTypeInfo>,
    http_client: Client, // Cliente HTTP
    fs_browser_port: u16, // Porta do node-fs-browser (configurada no main)
}

fn discover_nodes() -> Vec<NodeTypeInfo> {
    let mut discovered_nodes = Vec::new();
    let current_dir = std::env::current_dir().expect("Não consegui ler o diretório atual");
    let workspace_dir = &current_dir;
    println!("{} | 🟡 [Discovery] Procurando nodes em: {}", Utc::now().to_rfc3339(), workspace_dir.display());

    for entry in WalkDir::new(workspace_dir).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name == "target" || dir_name == "src" || dir_name.starts_with('.') || dir_name == "ndnm-core" || dir_name == "ndnm-brazil" { continue; }
                let config_path = path.join("config.yaml");
                match load_config(config_path.to_str().unwrap_or(""), path.to_str().unwrap_or("")) {
                    Ok((node_config, _)) => {
                        println!("{} | 🟢 [Discovery] Config válido encontrado para '{}'", Utc::now().to_rfc3339(), dir_name);
                        let node_type = node_config.node_type.clone().unwrap_or_else(|| dir_name.trim_start_matches("node-").to_string());
                        let label = node_config.label.clone().unwrap_or_else(|| node_type.clone());

                        let mut default_data = json!({
                            "label": label,
                            "inputsMode": node_config.inputs_mode.unwrap_or_else(|| "1".to_string()),
                            "inputsCount": node_config.initial_inputs_count.unwrap_or(1),
                            "outputsMode": node_config.outputs_mode.unwrap_or_else(|| "1".to_string()),
                            "outputsCount": node_config.initial_outputs_count.unwrap_or(1),
                            "value": if !node_config.input_fields.is_empty() { Some("") } else { None }
                        });

                        if !node_config.input_fields.is_empty() {
                            let fields_json = serde_json::to_value(&node_config.input_fields).unwrap_or(Value::Null);
                            if let Value::Object(ref mut map) = default_data {
                                map.insert("input_fields".to_string(), fields_json);
                            }
                        }

                        discovered_nodes.push(NodeTypeInfo {
                            r#type: node_type,
                            label: label,
                            default_data,
                        });
                    }
                    Err(_) => { }
                }
            }
        }
    }
    println!("{} | 🟡 [Discovery] Fim da busca. Nodes válidos encontrados: {}", Utc::now().to_rfc3339(), discovered_nodes.len());
    discovered_nodes.sort_by(|a, b| a.label.cmp(&b.label));
    discovered_nodes
}

async fn save_workspace(
    State(_state): State<Arc<AppState>>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let workspace_name = payload.get("name").and_then(|v| v.as_str());
    if workspace_name.is_none() {
        return (StatusCode::BAD_REQUEST, "Missing workspace name").into_response();
    }
    
    let workspace_name = workspace_name.unwrap();
    let workspace_dir = StdPath::new("workspaces");
    
    if !workspace_dir.exists() {
        if let Err(e) = fs::create_dir_all(workspace_dir) {
            println!("{} | 🔴 Erro ao criar pasta workspaces: {}", Utc::now().to_rfc3339(), e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response();
        }
    }
    
    let file_path = workspace_dir.join(format!("{}.json", workspace_name));
    
    match fs::write(&file_path, serde_json::to_string_pretty(&payload).unwrap_or_default()) {
        Ok(_) => {
            println!("{} | 💾 [Workspace] '{}' salvo em {:?}", 
                Utc::now().to_rfc3339(), workspace_name, file_path);
            (StatusCode::OK, axum::Json(json!({"status": "saved"}))).into_response()
        }
        Err(e) => {
            println!("{} | 🔴 [Workspace] Erro ao salvar: {}", Utc::now().to_rfc3339(), e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

async fn load_workspace(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let workspace_dir = StdPath::new("workspaces");
    let file_path = workspace_dir.join(format!("{}.json", name));
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(data) => {
                    println!("{} | 📂 [Workspace] '{}' carregado", Utc::now().to_rfc3339(), name);
                    (StatusCode::OK, axum::Json(data)).into_response()
                }
                Err(e) => {
                    println!("{} | 🔴 [Workspace] Erro ao parsear JSON: {}", Utc::now().to_rfc3339(), e);
                    (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response()
                }
            }
        }
        Err(e) => {
            println!("{} | 🔴 [Workspace] Não encontrado: {}", Utc::now().to_rfc3339(), e);
            (StatusCode::NOT_FOUND, format!("Workspace not found: {}", e)).into_response()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Cli::parse();
    let (mut brazil_config, config_path) = load_config(&args.config, env!("CARGO_MANIFEST_DIR"))?;
    println!("{} | 🟢 [WS Brazil] ndnm-brazil (Maestro) usando config: {}", Utc::now().to_rfc3339(), config_path.display());
    if let Some(p) = args.port { brazil_config.port = p; }
    if brazil_config.port == 0 { return Err(AppError::bad(format!("Porta inválida: {}", config_path.display()))); }
    
    // NOVO: Descobrir a porta do node-fs-browser (Assumindo que ele está na lista de nodes)
    let discovered_nodes = discover_nodes();
    let fs_browser_port = discovered_nodes.iter()
        .find(|n| n.r#type == "fsBrowser")
        .and_then(|_| load_config("../node-fs-browser/config.yaml", "")
            .ok().map(|(cfg, _)| cfg.port))
        .unwrap_or(3011); // Fallback para 3011, conforme definido no config.yaml

    println!("{} | 🟢 [WS Brazil] Node de Navegação de Arquivos (fsBrowser) na porta: {}", Utc::now().to_rfc3339(), fs_browser_port);


    let http_client = Client::new();
    let (tx, _) = broadcast::channel(100);
    
    let app_state = Arc::new(AppState { 
        tx, 
        known_nodes: discovered_nodes,
        http_client,
        fs_browser_port,
    });
    
    let cors = CorsLayer::permissive();
    
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .route("/workspace/save", post(save_workspace))
        .route("/workspace/load/:name", get(load_workspace))
        .with_state(app_state)
        .layer(cors);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], brazil_config.port));
    println!("{} | 🟢 [WS Brazil] ndnm-brazil ouvindo em {}", Utc::now().to_rfc3339(), addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn health_handler() -> impl IntoResponse { (StatusCode::OK, "Brazil is alive!") }

async fn ws_handler( ws: WebSocketUpgrade, State(state): State<Arc<AppState>> ) -> impl IntoResponse {
    println!("{} | 🟡 [WS Brazil] Novo cliente WebSocket tentando conectar...", Utc::now().to_rfc3339());
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("{} | 🟢 [WS Brazil] Cliente WebSocket CONECTADO!", Utc::now().to_rfc3339());
    let (mut sender, mut receiver) = socket.split();
    
    // MANDA O NODE_CONFIG INICIAL
    let config_msg = BrazilToFrontend::NodeConfig { payload: state.known_nodes.clone() };
    match serde_json::to_string(&config_msg) {
        Ok(json_str) => {
            if sender.send(Message::Text(json_str)).await.is_err() { return; }
            println!("{} | 🟢 [WS Brazil] Enviou NODE_CONFIG inicial ({} nodes).", Utc::now().to_rfc3339(), state.known_nodes.len());
        }
        Err(e) => { println!("{} | 🔴 [WS Brazil] Erro ao serializar NODE_CONFIG: {}", Utc::now().to_rfc3339(), e); return; }
    }
    
    let mut rx = state.tx.subscribe();
    // FIX Warning: O underscore _ indica que a variável não será usada
    let _state_clone_send = Arc::clone(&state); 
    
    // TASK DE ENVIO (BROADCAST)
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg_from_broadcast) = rx.recv().await {
            if !msg_from_broadcast.contains("\"type\":\"NODE_CONFIG\"") {
                if sender.send(Message::Text(msg_from_broadcast)).await.is_err() { break; }
            }
        }
    });
    
    let state_clone_recv = Arc::clone(&state);
    
    // TASK DE RECEBIMENTO (COMANDOS DO FRONT)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("{} | 🟢 [WS Brazil] Recebido do cliente: {}", Utc::now().to_rfc3339(), text);
                    
                    match serde_json::from_str::<FrontendToBrazil>(&text) {
                        // FIX Warning: unused variable: request_id
                        Ok(FrontendToBrazil::BrowsePath { path, request_id: _ }) => {
                            // CHAMA O NODE FS-BROWSER VIA HTTP
                            let node_url = format!("http://127.0.0.1:{}/run", state_clone_recv.fs_browser_port);
                            let input_body = FsBrowserInput { path };
                            
                            println!("{} | 🟡 [WS Brazil] Chamando node-fs-browser em: {}", Utc::now().to_rfc3339(), node_url);

                            let result = state_clone_recv.http_client
                                .post(&node_url)
                                .json(&input_body)
                                .send()
                                .await;
                            
                            match result {
                                Ok(resp) if resp.status().is_success() => {
                                    match resp.json::<FsBrowserOutput>().await {
                                        Ok(output) => {
                                            // ENVIA O RESULTADO PARA O FRONT VIA BROADCAST
                                            let fs_result = BrazilToFrontend::FsBrowseResult { 
                                                current_path: output.current_path, 
                                                entries: output.entries
                                            };
                                            if let Ok(json_str) = serde_json::to_string(&fs_result) {
                                                if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ }
                                            }
                                        }
                                        Err(e) => { 
                                            println!("{} | 🔴 [WS Brazil] Erro ao deserializar output do node: {}", Utc::now().to_rfc3339(), e); 
                                            // Envia erro para o frontend
                                            let error_msg = BrazilToFrontend::Echo { message: format!("ERRO DESERIALIZAÇÃO FS-BROWSER: {}", e) };
                                            if let Ok(json_str) = serde_json::to_string(&error_msg) { if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ } }
                                        }
                                    }
                                }
                                Ok(resp) => {
                                    // FIX E0382: Salva o status antes de chamar resp.text().await
                                    let status = resp.status(); 
                                    let error_text = resp.text().await.unwrap_or_default();
                                    println!("{} | 🔴 [WS Brazil] Erro HTTP do node (Status {}): {}", Utc::now().to_rfc3339(), status, error_text);
                                    
                                    // NOVO LOG: Enviar um erro de volta para o frontend para fins de debug
                                    let error_msg = BrazilToFrontend::Echo { 
                                        message: format!("ERRO FS-BROWSER: Status {} - {}", status, error_text) 
                                    };
                                    if let Ok(json_str) = serde_json::to_string(&error_msg) {
                                         if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ }
                                    }
                                }
                                Err(e) => { 
                                    println!("{} | 🔴 [WS Brazil] Falha ao conectar/enviar para o node: {}", Utc::now().to_rfc3339(), e); 
                                    
                                    // NOVO LOG: Enviar um erro de conexão para o frontend
                                    let error_msg = BrazilToFrontend::Echo { 
                                        message: format!("ERRO CONEXÃO FS-BROWSER: Node não encontrado na porta {}", state_clone_recv.fs_browser_port) 
                                    };
                                     if let Ok(json_str) = serde_json::to_string(&error_msg) {
                                         if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ }
                                    }
                                }
                            }
                        }
                        Ok(FrontendToBrazil::Echo { message }) => {
                            let echo_msg = BrazilToFrontend::Echo { message: format!("Brazil recebeu: {}", message) };
                            if let Ok(json_str) = serde_json::to_string(&echo_msg) {
                                println!("{} | 🟢 [WS Brazil] Enviando ECHO via broadcast: {}", Utc::now().to_rfc3339(), json_str);
                                if state_clone_recv.tx.send(json_str).is_err() { }
                            }
                        }
                        Ok(FrontendToBrazil::ExecutePlay { play_node_id, workspace_id, graph }) => {
                            println!("{} | 🚀 [WS Brazil] EXECUTE_PLAY recebido - play_node: {}, workspace: {}",
                                Utc::now().to_rfc3339(), play_node_id, workspace_id);

                            // Cria ExecutionEngine
                            let engine = execution::ExecutionEngine::new();

                            // Cria request de execução
                            let exec_request = execution::ExecutionRequest {
                                play_node_id: play_node_id.clone(),
                                workspace_id: workspace_id.clone(),
                                graph,
                            };

                            // Executa! (Fase 2: bloqueante - Fase 3: async + progress updates)
                            match engine.execute(exec_request).await {
                                Ok(result) => {
                                    println!("{} | ✅ [WS Brazil] Execução completa: run_id={}, nodes={}/{}",
                                        Utc::now().to_rfc3339(), result.run_id, result.executed_nodes, result.total_nodes);

                                    // Envia resultado pro frontend
                                    let complete_msg = BrazilToFrontend::ExecutionComplete {
                                        run_id: result.run_id,
                                        status: "completed".to_string(),
                                        total_nodes: result.total_nodes,
                                        executed_nodes: result.executed_nodes,
                                        cached_nodes: result.cached_nodes,
                                        duration_ms: result.duration_ms,
                                    };

                                    if let Ok(json_str) = serde_json::to_string(&complete_msg) {
                                        if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ }
                                    }
                                }
                                Err(error) => {
                                    println!("{} | ❌ [WS Brazil] Erro na execução: {}", Utc::now().to_rfc3339(), error);

                                    // Envia erro pro frontend
                                    let error_msg = BrazilToFrontend::ExecutionError {
                                        run_id: "unknown".to_string(), // TODO: Fase 3 - retornar run_id mesmo em erro
                                        error: error.clone(),
                                        failed_node: None, // TODO: Fase 3 - detectar node que falhou
                                    };

                                    if let Ok(json_str) = serde_json::to_string(&error_msg) {
                                        if state_clone_recv.tx.send(json_str).is_err() { /* ignore */ }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("{} | 🔴 [WS Brazil] Erro ao deserializar msg do front: {}", Utc::now().to_rfc3339(), e);
                        }
                    }
                }
                Message::Close(_) => { break; }
                _ => {}
            }
        }
    });
    
    // Seleção para terminar a tarefa se a outra falhar
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    println!("{} | 🟡 [WS Brazil] Conexão WebSocket finalizada.", Utc::now().to_rfc3339());
}