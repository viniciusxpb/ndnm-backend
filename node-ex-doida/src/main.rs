// node-ex-doida/src/main.rs
// Node de infraestrutura para logging e configurações

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router, Json,
};
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_http::cors::CorsLayer;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "3999")]
    port: u16,
}

#[derive(Debug, Clone)]
struct AppState {
    log_file_path: Arc<Mutex<PathBuf>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: Option<String>,
    level: String,      // "log", "error", "warn", "info", "debug"
    message: String,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct LogResponse {
    status: String,
    message: String,
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

async fn log_handler(
    State(state): State<AppState>,
    Json(entry): Json<LogEntry>,
) -> impl IntoResponse {
    let timestamp = entry.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339());

    // Formatar a mensagem de log
    let log_line = if let Some(data) = &entry.data {
        format!(
            "[{}] [{}] {} | Data: {}\n",
            timestamp,
            entry.level.to_uppercase(),
            entry.message,
            serde_json::to_string(data).unwrap_or_default()
        )
    } else {
        format!(
            "[{}] [{}] {}\n",
            timestamp,
            entry.level.to_uppercase(),
            entry.message
        )
    };

    // Printar no console do node-ex-doida também
    print!("{}", log_line);

    // Salvar no arquivo
    let log_file_path = state.log_file_path.lock().unwrap();
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path.as_path())
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(log_line.as_bytes()) {
                eprintln!("❌ Erro ao escrever no arquivo de log: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LogResponse {
                        status: "error".to_string(),
                        message: format!("Failed to write to log file: {}", e),
                    }),
                );
            }
            if let Err(e) = file.flush() {
                eprintln!("❌ Erro ao fazer flush do arquivo: {}", e);
            }
        }
        Err(e) => {
            eprintln!("❌ Erro ao abrir arquivo de log: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogResponse {
                    status: "error".to_string(),
                    message: format!("Failed to open log file: {}", e),
                }),
            );
        }
    }

    (
        StatusCode::OK,
        Json(LogResponse {
            status: "ok".to_string(),
            message: "Log saved".to_string(),
        }),
    )
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    // Define o caminho do arquivo de log na raiz do projeto
    // Como rodamos de ndnm-backend/, voltamos 1 nível
    let log_file_path = PathBuf::from("../logsFront.txt");

    println!("🔧 [node-ex-doida] Starting infrastructure node...");
    println!("📝 [node-ex-doida] Log file: {}", log_file_path.display());

    // Cria o arquivo se não existir
    if !log_file_path.exists() {
        match File::create(&log_file_path) {
            Ok(_) => println!("✅ [node-ex-doida] Log file created"),
            Err(e) => {
                eprintln!("❌ [node-ex-doida] Failed to create log file: {}", e);
                return;
            }
        }
    }

    let state = AppState {
        log_file_path: Arc::new(Mutex::new(log_file_path)),
    };

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/log", post(log_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("🟢 [node-ex-doida] Listening on {}", addr);
    println!("📡 [node-ex-doida] POST /log - Receive frontend logs");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
