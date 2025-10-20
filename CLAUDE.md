# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ndnm (No deps, no masters)** is a local-first, open-source visual node orchestration system - a workflow engine where **any module (node) in any language** can be plugged in, executed, and visually connected to others.

**Key Philosophy:**
- **Language-agnostic nodes:** Nodes can be written in Rust, Python, JavaScript, or any language that can expose HTTP endpoints
- **No dependencies, no masters:** Minimize external dependencies, keep code resilient and self-contained
- **Local-first:** Everything runs locally as independent microservices

**Tech Stack:**
- Backend orchestrator: Rust (ndnm-brazil)
- Node library (optional): Rust (ndnm-core) - but nodes can be in ANY language
- Frontend: React (cross-platform: Windows, Mac, Linux)

**Location:** `C:\Projetos\ndnm\ndnm-backend\`

## Development Commands

### Building and Running

```bash
# Run a specific Rust node (e.g., node-sum on port 3000)
cargo run -p node-sum

# Run the orchestrator (ndnm-brazil on port 3100)
cargo run -p ndnm-brazil

# Run with custom port
cargo run -p node-sum -- -p 3001

# Run with custom config
cargo run -p node-sum -- --config custom-config.yaml

# Build all Rust workspace members
cargo build

# Build specific node
cargo build -p node-fs-browser
```

### Testing Nodes (Any Language)

**All nodes must expose these HTTP endpoints regardless of language:**
- `GET /health` → `{"status": "ok"}`
- `POST /run` → Accepts JSON input, returns JSON output

Use PowerShell's `Invoke-RestMethod` or curl:

**Health check:**
```powershell
Invoke-RestMethod -Uri http://localhost:3000/health
```

**Execute node logic:**
```powershell
# Example: node-sum
Invoke-RestMethod -Uri http://localhost:3000/run -Method Post -ContentType 'application/json' -Body '{"variables":[10, 30, 2]}'

# Example: node-fixed-value
Invoke-RestMethod -Uri http://localhost:3010/run -Method Post -ContentType 'application/json' -Body '{"value":"hello world"}'
```

**With curl (Linux/macOS/WSL):**
```bash
curl --header "Content-Type: application/json" --request POST --data '{"variables":[10, 30, 2]}' http://localhost:3000/run
```

## Architecture

### Language-Agnostic Design

The system is designed so that **nodes can be written in any language**. The only requirements are:

1. Expose HTTP server on a configured port
2. Implement `GET /health` endpoint
3. Implement `POST /run` endpoint that accepts/returns JSON
4. Have a `config.yaml` file for discovery

**Current implementations:**
- **Rust nodes:** Use ndnm-core library (optional convenience)
- **Python nodes:** Examples exist (e.g., `node-clip-text-encode-py/`) - not fully functional yet
- **Future:** JavaScript, Go, or any language with HTTP capabilities

### Three-Layer System

1. **ndnm-core** (Optional Rust Library - `ndnm-core/`)
   - Convenience toolkit for Rust nodes
   - Provides HTTP server setup, routing, configuration loading, error handling
   - Defines the `Node` trait for type-safe implementation
   - **Not required** - nodes in other languages implement the HTTP contract directly

2. **node-*** (Independent HTTP Microservices)
   - Each node runs on its own port as an independent process
   - Can be written in **any language** (Rust, Python, JavaScript, etc.)
   - Examples:
     - `node-sum/` - Rust implementation
     - `node-fs-browser/` - Rust implementation
     - `node-clip-text-encode-py/` - Python implementation (in progress)
   - Business logic separated from HTTP layer

3. **ndnm-brazil** (Orchestrator - `ndnm-brazil/`)
   - Central hub server (port 3100) written in Rust
   - Discovers nodes by scanning directories for `config.yaml` files
   - Manages WebSocket connections to React frontend
   - Proxies requests from frontend to nodes via HTTP (language-agnostic)
   - Persists and loads workflow definitions to/from `workspaces/` directory

### Directory Structure

```
ndnm-backend/
├── Cargo.toml                    # Rust workspace root (only for Rust nodes)
├── ndnm-core/                    # Optional Rust library
│   └── src/
│       ├── lib.rs                # Main exports
│       ├── node/mod.rs           # Node trait definition
│       ├── server/               # HTTP server setup
│       ├── config/               # YAML config loading
│       ├── runner/               # CLI argument parsing
│       └── error/                # Error types & HTTP responses
├── ndnm-brazil/                  # Orchestrator (Rust)
│   ├── config.yaml               # Port 3100
│   └── src/main.rs
├── node-sum/                     # Rust node example
│   ├── config.yaml               # Port 3000
│   └── src/
│       ├── main.rs               # Node trait impl + entry point
│       └── domain.rs             # Pure business logic
├── node-fs-browser/              # Rust: File system navigation
├── node-fixed-value/             # Rust: Constant value node
├── node-clip-text-encode-py/     # Python node (in progress)
├── node-subtract/                # Rust node
├── node-load-checkpoint/         # Rust node
├── models/                       # ML model files
└── workspaces/                   # Saved workflow JSON files
```

### Node Contract (Any Language)

**Every node must:**

1. **Expose HTTP server** on the port specified in `config.yaml`

2. **Implement GET /health:**
   ```json
   Response: {"status": "ok"}
   ```

3. **Implement POST /run:**
   ```
   Request Content-Type: application/json
   Request Body: (node-specific JSON structure)

   Response Content-Type: application/json
   Response Body: (node-specific JSON structure)
   ```

4. **Have config.yaml in node directory:**
   ```yaml
   port: 3000                    # HTTP port for this node
   label: "➕ Somar"            # Display name in React UI
   node_type: "add"             # Type identifier
   inputs_mode: "n"             # "0" (none), "1" (fixed), "n" (dynamic)
   initial_inputs_count: 1
   outputs_mode: "1"            # "0" (none), "1" (fixed), "n" (dynamic)
   initial_outputs_count: 1
   input_fields:                # UI form fields for React frontend
     - name: "value"
       type: "text"
   ```

### Communication Flow

**Example: File browsing**

1. **React Frontend** sends WebSocket message to ndnm-brazil (port 3100):
   ```json
   {"type": "BROWSE_PATH", "path": "C:\\Users", "request_id": "abc123"}
   ```

2. **ndnm-brazil** proxies HTTP POST to node-fs-browser (port 3011):
   ```json
   POST http://localhost:3011/run
   {"value": "C:\\Users"}
   ```

3. **node-fs-browser** (Rust) scans directory, returns:
   ```json
   {
     "current_path": "C:\\Users",
     "entries": [
       {"name": "Public", "is_dir": true, "path": "C:\\Users\\Public", ...},
       {"name": "..", "is_dir": true, "path": "C:\\", ...}
     ]
   }
   ```

4. **ndnm-brazil** broadcasts result to React frontend via WebSocket

**The beauty:** Step 3 could be implemented in Python, JavaScript, or any language - ndnm-brazil doesn't care as long as the HTTP contract is met.

## Creating Nodes in Different Languages

### Creating a Rust Node (Using ndnm-core)

1. **Create directory structure:**
   ```
   node-multiply/
   ├── Cargo.toml
   ├── config.yaml
   └── src/
       ├── main.rs
       └── domain.rs
   ```

2. **Write Cargo.toml:**
   ```toml
   [package]
   name = "node-multiply"
   version = "0.1.0"
   edition = "2021"
   default-run = "node-multiply"

   [dependencies]
   ndnm-core = { path = "../ndnm-core" }
   serde = { version = "1", features = ["derive"] }
   tokio = { version = "1", features = ["full"] }
   ```

3. **Write config.yaml:**
   ```yaml
   port: 3012
   label: "✕ Multiplicar"
   node_type: "multiply"
   inputs_mode: "n"
   initial_inputs_count: 2
   outputs_mode: "1"
   initial_outputs_count: 1
   ```

4. **Implement in main.rs:**
   ```rust
   use ndnm_core::{async_trait, AppError, Node};
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Deserialize)]
   pub struct Input {
       variables: Vec<i64>,
   }

   #[derive(Debug, Serialize)]
   pub struct Output {
       response: i64,
   }

   #[derive(Default)]
   pub struct MultiplyNode;

   #[async_trait]
   impl Node for MultiplyNode {
       type Input = Input;
       type Output = Output;

       async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
           let result = input.variables.iter().product::<i64>();
           Ok(Output { response: result })
       }
   }

   #[tokio::main]
   async fn main() -> Result<(), AppError> {
       ndnm_core::run_node(
           MultiplyNode::default(),
           "node-multiply",
           "Multiplies a list of integers",
           env!("CARGO_MANIFEST_DIR"),
       ).await
   }
   ```

5. **Add to workspace in root Cargo.toml:**
   ```toml
   [workspace]
   members = [
       ...
       "node-multiply",
   ]
   ```

6. **Run:**
   ```bash
   cargo run -p node-multiply
   ```

### Creating a Python Node (Manual HTTP Implementation)

1. **Create directory structure:**
   ```
   node-sentiment-analysis/
   ├── config.yaml
   ├── requirements.txt
   └── main.py
   ```

2. **Write config.yaml:**
   ```yaml
   port: 3013
   label: "🎭 Análise de Sentimento"
   node_type: "sentimentAnalysis"
   inputs_mode: "1"
   initial_inputs_count: 1
   outputs_mode: "1"
   initial_outputs_count: 1
   input_fields:
     - name: "text"
       type: "text"
   ```

3. **Write main.py:**
   ```python
   from flask import Flask, request, jsonify
   import yaml

   app = Flask(__name__)

   # Load config
   with open('config.yaml', 'r') as f:
       config = yaml.safe_load(f)
       PORT = config['port']

   @app.route('/health', methods=['GET'])
   def health():
       return jsonify({"status": "ok"})

   @app.route('/run', methods=['POST'])
   def run():
       try:
           data = request.get_json()
           text = data.get('text', '')

           # Your business logic here
           sentiment = analyze_sentiment(text)

           return jsonify({"sentiment": sentiment})
       except Exception as e:
           return jsonify({
               "status": "error",
               "error": {
                   "code": "INTERNAL",
                   "message": str(e)
               }
           }), 500

   def analyze_sentiment(text):
       # Your Python ML logic here
       return "positive"

   if __name__ == '__main__':
       app.run(host='0.0.0.0', port=PORT)
   ```

4. **Run:**
   ```bash
   pip install -r requirements.txt
   python main.py
   ```

**ndnm-brazil will discover it automatically** as long as `config.yaml` exists in the directory!

### Creating a JavaScript/Node.js Node

1. **Create directory structure:**
   ```
   node-text-transform/
   ├── config.yaml
   ├── package.json
   └── index.js
   ```

2. **Write config.yaml:**
   ```yaml
   port: 3014
   label: "📝 Transformar Texto"
   node_type: "textTransform"
   inputs_mode: "1"
   outputs_mode: "1"
   ```

3. **Write index.js:**
   ```javascript
   const express = require('express');
   const yaml = require('js-yaml');
   const fs = require('fs');

   const config = yaml.load(fs.readFileSync('config.yaml', 'utf8'));
   const app = express();
   app.use(express.json());

   app.get('/health', (req, res) => {
       res.json({ status: 'ok' });
   });

   app.post('/run', (req, res) => {
       try {
           const { text, transform } = req.body;
           const result = transformText(text, transform);
           res.json({ result });
       } catch (error) {
           res.status(500).json({
               status: 'error',
               error: {
                   code: 'INTERNAL',
                   message: error.message
               }
           });
       }
   });

   function transformText(text, transform) {
       // Your business logic
       return text.toUpperCase();
   }

   app.listen(config.port, '0.0.0.0', () => {
       console.log(`Node listening on port ${config.port}`);
   });
   ```

## Frontend Integration

**React Frontend** communicates with the system via:

1. **WebSocket connection to ndnm-brazil (port 3100)**
   - Receives node configuration for UI rendering
   - Sends execution commands
   - Receives real-time results

2. **Cross-platform support:**
   - Windows
   - macOS
   - Linux

3. **UI generation from config.yaml:**
   - Frontend reads `input_fields` to generate form inputs
   - Reads `inputs_mode`/`outputs_mode` to render connection points
   - Uses `label` for display name

## Key Architectural Patterns

### Language Agnosticism
- **Core principle:** Nodes are black boxes that speak HTTP and JSON
- **Discovery:** ndnm-brazil scans for `config.yaml` files, doesn't care about language
- **Communication:** Pure HTTP POST/GET - works with any language
- **Benefits:** Use the right language for each node (Python for ML, Rust for performance, JS for web APIs)

### Separation of Concerns
- **ndnm-core** (Rust library) is optional convenience, not required
- **ndnm-brazil** (orchestrator) is language-agnostic in how it talks to nodes
- **Each node** is an independent process with its own dependencies
- **React frontend** is completely decoupled from backend implementation

### Configuration-Driven UI
The React frontend generates UI based on `config.yaml`:
- No frontend code changes needed for new nodes
- Consistent UX across all nodes
- Dynamic form generation from `input_fields`

### Microservices Architecture
- Each node is an independent HTTP service
- Nodes can crash/restart without affecting others
- Easy to develop and test nodes in isolation
- No shared state between nodes

## Technology Stack

**Backend (Orchestrator):**
- Rust (ndnm-brazil)
- Tokio (async runtime)
- Axum (HTTP + WebSocket)
- Serde (JSON/YAML)

**Backend (Rust Nodes - Optional):**
- ndnm-core library
- Axum 0.7 (HTTP)
- Tokio (async)

**Backend (Python Nodes - Examples):**
- Flask or FastAPI for HTTP
- Any Python ML libraries

**Frontend:**
- React (cross-platform: Windows, Mac, Linux)
- WebSocket client for real-time communication

**Common:**
- HTTP/JSON for inter-process communication
- YAML for configuration

## Important File Locations

**Core Infrastructure:**
- `ndnm-core/src/node/mod.rs` - Rust Node trait definition
- `ndnm-core/src/server/router.rs` - Generic route setup for Rust nodes
- `ndnm-brazil/src/main.rs` - Orchestrator with WebSocket and node discovery

**Node Examples:**
- `node-fs-browser/src/domain.rs` - File system navigation (Rust)
- `node-sum/src/main.rs` - Simple math node (Rust)
- `node-clip-text-encode-py/` - Python node example (in progress)

**Data:**
- `workspaces/` - Persisted workflow JSON files
- `models/` - ML model files

## Notes from Development Philosophy

- **Language-agnostic by design:** Don't restrict creativity to one language
- **Local-first:** Everything runs on user's machine, no cloud dependencies
- **Minimal dependencies:** Each node manages its own deps, no global lock-in
- **PowerShell for Windows development:** Use `Invoke-RestMethod`, `New-Item`
- **Cross-platform React frontend:** Must work on Windows, Mac, and Linux
- **HTTP as universal interface:** Simple, debuggable, language-independent

## Node Discovery Process

When ndnm-brazil starts:

1. Scans workspace directory (parent of ndnm-brazil)
2. Looks for subdirectories (skips `target/`, `src/`, `.git`, `ndnm-core`, `ndnm-brazil`)
3. Tries to load `config.yaml` from each directory
4. If `config.yaml` exists and is valid, adds to discovered nodes
5. Sends node list to React frontend via WebSocket `NODE_CONFIG` message
6. Frontend renders available nodes in UI

**This means:** Drop any HTTP service with a `config.yaml` in the workspace, and it becomes available in the UI!
