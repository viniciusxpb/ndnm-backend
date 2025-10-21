# TESTE DA FASE 2 - EXECUTION ENGINE

## Pré-requisitos

Certifique-se que os nodes estão rodando nas portas corretas:
- `node-sum` → porta 3000
- `node-subtract` → porta 3001
- `ndnm-brazil` → porta 3100

---

## PASSO 1: Rodar os Nodes

Abra **3 terminais separados** e rode:

### Terminal 1 - node-sum
```powershell
cd C:\Projetos\ndnm\ndnm-backend
cargo run -p node-sum
```

Aguarde até ver: `node-sum ouvindo na porta 3000`

### Terminal 2 - node-subtract
```powershell
cd C:\Projetos\ndnm\ndnm-backend
cargo run -p node-subtract
```

Aguarde até ver: `node-subtract ouvindo na porta 3001`

### Terminal 3 - ndnm-brazil
```powershell
cd C:\Projetos\ndnm\ndnm-backend
cargo run -p ndnm-brazil
```

Aguarde até ver: `listening on http://0.0.0.0:3100`

---

## PASSO 2: Testar Nodes Individualmente (Opcional mas Recomendado)

Abra um **4º terminal** para testes:

### Testar node-sum
```powershell
Invoke-RestMethod -Uri http://localhost:3000/run -Method Post -ContentType 'application/json' -Body '{"variables":[10, 20, 5]}'
```

**Resultado esperado:** `{"response": 35}`

### Testar node-subtract
```powershell
Invoke-RestMethod -Uri http://localhost:3001/run -Method Post -ContentType 'application/json' -Body '{"variables":[100, 30, 5]}'
```

**Resultado esperado:** `{"response": 65}`

---

## PASSO 3: Testar Execução Completa (E2E)

**ATENÇÃO:** Como WebSocket é mais complexo de testar via PowerShell, vou te dar 2 opções:

### OPÇÃO A: Teste Simplificado (Script PowerShell com WebSocket)

Crie um arquivo `test_ws.ps1` com:

```powershell
# test_ws.ps1
$json = Get-Content "C:\Projetos\ndnm\ndnm-backend\test_execution.json" -Raw

# Conecta ao WebSocket
$ws = New-Object System.Net.WebSockets.ClientWebSocket
$uri = [System.Uri]::new("ws://localhost:3100/ws")
$cts = New-Object System.Threading.CancellationTokenSource

try {
    $connectTask = $ws.ConnectAsync($uri, $cts.Token)
    $connectTask.Wait()
    Write-Host "✅ Conectado ao WebSocket"

    # Envia JSON
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
    $segment = New-Object System.ArraySegment[byte] -ArgumentList @(,$bytes)
    $sendTask = $ws.SendAsync($segment, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $cts.Token)
    $sendTask.Wait()
    Write-Host "📤 Mensagem enviada!"

    # Aguarda resposta
    $buffer = New-Object byte[] 4096
    $segment = New-Object System.ArraySegment[byte] -ArgumentList @(,$buffer)
    $receiveTask = $ws.ReceiveAsync($segment, $cts.Token)
    $receiveTask.Wait()

    $response = [System.Text.Encoding]::UTF8.GetString($buffer, 0, $receiveTask.Result.Count)
    Write-Host "📥 Resposta recebida:"
    Write-Host $response

} finally {
    $ws.Dispose()
    $cts.Dispose()
}
```

Execute:
```powershell
.\test_ws.ps1
```

### OPÇÃO B: Teste Manual (Mais Simples - Usar Postman/Insomnia/WebSocket Client)

1. Instale uma extensão WebSocket pro navegador OU use Postman
2. Conecte em: `ws://localhost:3100/ws`
3. Envie o conteúdo de `test_execution.json` (copie e cole)

---

## OPÇÃO C: Teste Direto pelo Executor (Bypass WebSocket - Mais Fácil!)

Crie um arquivo de teste Rust:

### Criar: `ndnm-brazil/tests/test_execution.rs`

```rust
// ndnm-brazil/tests/test_execution.rs
use ndnm_brazil::execution::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_simple_execution() {
    // Cria grafo: sum -> subtract -> play
    let graph = WorkflowGraph {
        nodes: vec![
            GraphNode {
                id: "sum-1".to_string(),
                node_type: "add".to_string(),
                port: 3000,
                label: "Somar".to_string(),
                data: {
                    let mut map = HashMap::new();
                    map.insert("variables".to_string(), serde_json::json!([10, 20, 5]));
                    map
                },
            },
            GraphNode {
                id: "subtract-1".to_string(),
                node_type: "subtract".to_string(),
                port: 3001,
                label: "Subtrair".to_string(),
                data: {
                    let mut map = HashMap::new();
                    map.insert("variables".to_string(), serde_json::json!([100, 30, 5]));
                    map
                },
            },
            GraphNode {
                id: "play-1".to_string(),
                node_type: "playButton".to_string(),
                port: 3020,
                label: "Play".to_string(),
                data: HashMap::new(),
            },
        ],
        connections: vec![
            Connection {
                from_node_id: "sum-1".to_string(),
                from_output_index: 0,
                to_node_id: "subtract-1".to_string(),
                to_input_index: 0,
            },
            Connection {
                from_node_id: "subtract-1".to_string(),
                from_output_index: 0,
                to_node_id: "play-1".to_string(),
                to_input_index: 0,
            },
        ],
    };

    let request = ExecutionRequest {
        play_node_id: "play-1".to_string(),
        workspace_id: "test".to_string(),
        graph,
    };

    let engine = ExecutionEngine::new();
    let result = engine.execute(request).await;

    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert_eq!(exec_result.executed_nodes, 2); // sum + subtract (play é pulado)
    println!("✅ Teste passou! run_id: {}", exec_result.run_id);
}
```

**PROBLEMA:** ndnm-brazil não exporta o módulo `execution` publicamente ainda. Precisa adicionar no `lib.rs`.

---

## OPÇÃO D: Teste Mais Simples - Criar um Binário de Teste

Crie `ndnm-brazil/examples/test_exec.rs`:

```rust
// ndnm-brazil/examples/test_exec.rs
// Para rodar: cargo run -p ndnm-brazil --example test_exec

use std::collections::HashMap;

// Simula uma execução manual
#[tokio::main]
async fn main() {
    println!("🚀 Testando Execution Engine...");
    println!("⚠️  IMPORTANTE: Certifique-se que node-sum (3000) e node-subtract (3001) estão rodando!");

    // TODO: Código aqui se precisar testar direto
    // Por enquanto, use os testes via WebSocket ou Postman

    println!("✅ Use os testes WebSocket documentados em TESTE_FASE2.md");
}
```

---

## O QUE ESPERAR NOS LOGS

### Logs do ndnm-brazil (Terminal 3):
```
🚀 Iniciando execução: run_id=run_2024-10-20_23-45-12_abc123
   Play node: play-1
   Ordem de execução: ["sum-1", "subtract-1", "play-1"]
   Total de nodes: 3
⏭️  Pulando Play node: play-1
⚙️  Executando node: sum-1 (➕ Somar)
   ✅ Sucesso: sum-1 em 15ms
⚙️  Executando node: subtract-1 (➖ Subtrair)
   ✅ Sucesso: subtract-1 em 12ms
🎉 Execução completa: run_id=run_2024-10-20_23-45-12_abc123
   Nodes executados: 2
   Duração total: 27ms
```

### Resposta WebSocket Esperada:
```json
{
  "type": "EXECUTION_COMPLETE",
  "run_id": "run_2024-10-20_23-45-12_abc123",
  "status": "completed",
  "total_nodes": 3,
  "executed_nodes": 2,
  "cached_nodes": 0,
  "duration_ms": 27
}
```

---

## TROUBLESHOOTING

### Erro: "Node não encontrado"
- Verifique se node-sum e node-subtract estão rodando
- Use `netstat -ano | findstr :3000` pra ver se a porta tá ocupada

### Erro: "Connection refused"
- ndnm-brazil pode não ter iniciado ainda
- Aguarde os logs `listening on http://0.0.0.0:3100`

### Erro de compilação
- Rode `cargo clean` e tente novamente
- Verifique se todas as dependências foram instaladas (`cargo build`)

---

## RESUMO DO QUE TESTAR

✅ **Teste 1:** Nodes individuais funcionam (node-sum, node-subtract)
✅ **Teste 2:** ndnm-brazil inicia sem erros
✅ **Teste 3:** Execução E2E via WebSocket (OPÇÃO B é a mais fácil - Postman/Insomnia)

**Resultado esperado:**
- Logs mostram execução sequencial: sum-1 → subtract-1
- Resposta WebSocket com status "completed"
- 2 nodes executados (Play é pulado)

---

## PRÓXIMOS PASSOS (Após Teste)

Se tudo funcionar:
1. ✅ Fase 2 está COMPLETA!
2. Próxima: Fase 3 - Sistema de Cache com Hash
3. Depois: Fase 4 - Play com Output (Cascata)

**BOA SORTE! 🔥🚀**
