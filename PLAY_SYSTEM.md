# PLAY SYSTEM - Documentação Completa

## Visão Geral

O **Play System** é o motor de execução do ndnm. Diferente do ComfyUI (onde há um botão global de "play"), no ndnm **cada Play é um node visual no grafo**.

### Por que isso é revolucionário?

- **Execução granular**: Você escolhe exatamente qual parte do grafo executar
- **Múltiplos triggers**: Um grafo pode ter vários Plays com diferentes gatilhos (botão, timer, webhook)
- **Cascata de execuções**: Um Play pode disparar outro Play (Play tem output!)
- **Cache inteligente**: Só re-executa o que mudou (baseado em hash)

---

## Conceitos Fundamentais

### 1. Play é um Node (não um botão global)

```
[Node A] → [Node B] → [Play Button] → [Node C]
                           ↓
                      Executa apenas
                      A → B (sua cadeia)
```

No grafo acima, o Play Button **só executa A e B** (nodes conectados a ele). Node C não é executado.

### 2. Tipos de Play

| Tipo | Trigger | Tem Input? | Exemplo de Uso |
|------|---------|------------|----------------|
| **PlayButton** | Clique manual | Sim | Executar workflow sob demanda |
| **PlayTimer** | Intervalo de tempo | Opcional | Gerar relatório a cada hora |
| **PlayWebhook** | POST HTTP externo | Sim | Reagir a eventos externos |
| **PlayFileWatcher** | Arquivo modificado | Sim | Reprocessar quando dados mudam |

### 3. Sistema de Cache Inteligente

**Problema:** Re-executar um grafo de 50 nodes porque mudou só o último é desperdício.

**Solução:** Sistema baseado em **hash de inputs**.

#### Estrutura de Cache

```
workspaces/runs/
├── run_2024-10-20_22-30-15_abc123/
│   ├── node-sum-1/
│   │   ├── output.json           # Resultado da execução
│   │   ├── input-hash.txt        # Hash dos inputs (ex: "a3f2b9...")
│   │   └── output-hash.txt       # Hash do output
│   ├── node-multiply-2/
│   │   ├── output.json
│   │   ├── input-hash.txt
│   │   └── output-hash.txt
│   └── metadata.json             # Info geral da execução
└── run_2024-10-20_23-15-42_def456/  # Outra execução
```

#### Fluxo de Re-execução

```
1. User clica em Play pela segunda vez
2. Para cada node na cadeia:
   a. Calcula hash dos inputs atuais
   b. Busca último run-id desse node
   c. Compara hash atual com input-hash.txt
   d. Se IGUAL → Pula execução, usa cache anterior ✅
   e. Se DIFERENTE → Executa node, salva novo cache ⚙️
3. Economia: Se só mudou último node, executa 1 de 50!
```

**Exemplo:**

```
Primeira execução:
[A: val=10] → [B: *2] → [C: +5] = 25
Cache: run_001/
  ├── node-A/ (input-hash: "xyz123")
  ├── node-B/ (input-hash: "abc456")
  └── node-C/ (input-hash: "def789")

Segunda execução (mudou A para val=10 novamente):
Hash de A = "xyz123" → IGUAL! Usa cache
Hash de B = "abc456" → IGUAL! Usa cache
Hash de C = "def789" → IGUAL! Usa cache
Resultado: NENHUMA execução! ⚡

Terceira execução (mudou A para val=20):
Hash de A = "zzz999" → DIFERENTE! Executa A
Hash de B = "www888" → DIFERENTE! (porque A mudou) Executa B
Hash de C = "ttt777" → DIFERENTE! (porque B mudou) Executa C
Cache: run_002/
```

**Por que espaço em disco não é problema?**
- SSD de 1TB custa ~R$300
- Cache de 1 execução completa de ML: ~5GB
- 200 execuções = 1TB
- Limpar caches antigos é trivial

---

## Execução: Resolução de Dependências

### Algoritmo: Depth-First Sequencial

**Grafo de exemplo:**

```
      [A: id=1]
       /     \
  [B: id=2] [D: id=3]
       \     /
      [C: id=4]
         ↓
    [Play: id=5]
```

**Ordem de execução quando Play é disparado:**

```
1. Play identifica: "Preciso de C"
2. C identifica: "Preciso de B e D"
3. B identifica: "Preciso de A"
4. D identifica: "Preciso de A"

Ordem final: A → B → D → C
(Depth-first, e quando há bifurcação usa ordem de ID)
```

### Por que Sequencial (não Paralelo)?

1. **Evita race conditions** (dois nodes escrevendo no mesmo arquivo)
2. **Evita sobrecarga** (ML nodes consomem MUITA RAM/GPU)
3. **Debug mais fácil** (logs lineares, não entrelaçados)
4. **Cache consistente** (não precisa de locks)

### Múltiplos Outputs de um Node

```
[A: id=1]
 /  |  \
[B] [D] [E]
id=2 id=4 id=3
```

**Ordem de execução dos outputs de A:**
- **B (id=2) → E (id=3) → D (id=4)**
- Ordenado por ID do node (= ordem de criação)
- Determinístico e previsível

---

## Play com Output: Cascata de Execuções

### Exemplo: Pipeline de Processamento

```
[Timer: 5min] → [Process Images] → [Webhook: Notifica] → [Generate Report]
  (Play A)         (nodes...)          (Play B)            (nodes...)
```

**Fluxo:**

1. **Play A (Timer)** dispara a cada 5 minutos
   - Executa sua cadeia de nodes (Process Images)
   - Salva cache em `run-A-001/`
   - Output do Play A vai pra Play B

2. **Play B (Webhook)** recebe output de Play A
   - Cria NOVA execução independente
   - Salva cache em `run-B-001/` (pasta separada!)
   - Executa sua cadeia (Generate Report)

**Estrutura de pastas:**

```
workspaces/runs/
├── run-A-001/  ← Primeira execução do Timer
│   └── (nodes de Process Images)
├── run-B-001/  ← Disparado por run-A-001
│   └── (nodes de Generate Report)
├── run-A-002/  ← Segunda execução do Timer (5 min depois)
│   └── (nodes de Process Images)
└── run-B-002/  ← Disparado por run-A-002
    └── (nodes de Generate Report)
```

**Importante:** Cada Play = Cache isolado!

---

## Comunicação Frontend ↔ Brazil

### Mensagens WebSocket

#### 1. Frontend → Brazil: Disparar Play

```json
{
  "type": "EXECUTE_PLAY",
  "play_node_id": "play-btn-1",
  "workspace_id": "workspace-123"
}
```

#### 2. Brazil → Frontend: Status de Execução

```json
{
  "type": "EXECUTION_STATUS",
  "run_id": "run_2024-10-20_22-30-15_abc123",
  "status": "executing",
  "current_node": {
    "id": "node-sum-2",
    "label": "➕ Somar",
    "status": "executing"
  },
  "progress": {
    "completed": ["node-A-1", "node-B-2"],
    "current": "node-sum-3",
    "remaining": ["node-D-4", "node-C-5"]
  }
}
```

#### 3. Brazil → Frontend: Execução Completa

```json
{
  "type": "EXECUTION_COMPLETE",
  "run_id": "run_2024-10-20_22-30-15_abc123",
  "status": "success",
  "total_nodes": 5,
  "executed_nodes": 3,
  "cached_nodes": 2,
  "duration_ms": 1523,
  "cache_path": "workspaces/runs/run_2024-10-20_22-30-15_abc123"
}
```

#### 4. Brazil → Frontend: Erro de Execução

```json
{
  "type": "EXECUTION_ERROR",
  "run_id": "run_2024-10-20_22-30-15_abc123",
  "failed_node": {
    "id": "node-sum-3",
    "label": "➕ Somar"
  },
  "error": {
    "code": "INTERNAL",
    "message": "Division by zero"
  }
}
```

---

## Implementação por Fases

### Fase 1: PlayButton (ATUAL - Começar aqui!)

**Escopo:**
- Node simples que responde a clique manual
- Recebe mensagem WebSocket do frontend
- Executa nodes conectados sequencialmente
- SEM cache ainda (executa tudo sempre)
- SEM resolução complexa de dependências (só lista linear)

**Objetivo:** Provar conceito básico

**Arquivos:**
```
node-play-button/
├── Cargo.toml
├── config.yaml
└── src/
    ├── main.rs      # Entry point + Node trait
    └── domain.rs    # Lógica de execução (será simples nessa fase)
```

### Fase 2: Cache Inteligente

**Adicionar:**
- Geração de run-id
- Cálculo de hash de inputs
- Salvamento de cache
- Comparação de hash pra pular execução

### Fase 3: Resolução de Dependências

**Adicionar:**
- Algoritmo depth-first
- Resolução de múltiplos inputs
- Ordenação por ID de node

### Fase 4: Play com Output

**Adicionar:**
- Output do Play node
- Disparar próximo Play ao finalizar

### Fase 5: Outros Tipos de Play

**Adicionar:**
- PlayTimer (timer interno)
- PlayWebhook (escuta porta HTTP)
- PlayFileWatcher (monitora arquivo)

---

## Estrutura de Código (Fase 1)

### node-play-button/config.yaml

```yaml
port: 3100
label: "▶️ Play"
node_type: "playButton"
inputs_mode: "1"           # Tem input (conecta em outros nodes)
initial_inputs_count: 1
outputs_mode: "0"          # SEM output (por enquanto, Fase 4 adiciona)
initial_outputs_count: 0
```

### node-play-button/src/main.rs (Simplificado - Fase 1)

```rust
// Fase 1: Só recebe clique e retorna "OK, executei!"
// NÃO executa nodes ainda (isso vai pro Brazil)

#[derive(Debug, Deserialize)]
pub struct Input {
    action: String,  // "execute"
}

#[derive(Debug, Serialize)]
pub struct Output {
    status: String,  // "started"
    message: String,
}

#[async_trait]
impl Node for PlayButtonNode {
    type Input = Input;
    type Output = Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, AppError> {
        if input.action == "execute" {
            // Fase 1: Só confirma que recebeu
            // Fase 2+: Aqui vai disparar execução via Brazil
            Ok(Output {
                status: "started".to_string(),
                message: "Execution started".to_string(),
            })
        } else {
            Err(AppError::BadRequest("Invalid action".to_string()))
        }
    }
}
```

### ndnm-brazil/src/execution/ (Novo módulo - Fase 1)

```rust
// execution/mod.rs - Motor de execução

pub struct ExecutionEngine {
    // Fase 1: Bem simples
    // Fase 2+: Adiciona cache, hash, etc.
}

impl ExecutionEngine {
    pub async fn execute_play(&self, play_node_id: &str) -> Result<(), AppError> {
        // Fase 1:
        // 1. Identifica nodes conectados ao Play
        // 2. Executa sequencialmente (lista linear, sem resolver deps)
        // 3. Envia progresso via WebSocket

        // Fase 2+: Adiciona cache, hash, etc.
    }
}
```

---

## Exemplos de Uso

### Exemplo 1: Workflow Simples

```
[Node Sum] → [Node Multiply] → [Play Button]
  (id=1)        (id=2)            (id=3)
```

**Ação:** User clica no Play Button

**Resultado:**
```
1. Frontend envia: {"type": "EXECUTE_PLAY", "play_node_id": "3"}
2. Brazil identifica: "Play 3 conectado em 2"
3. Brazil identifica: "Node 2 conectado em 1"
4. Brazil executa:
   a. Node 1 (Sum) → resultado: 15
   b. Node 2 (Multiply) → recebe 15 → resultado: 30
5. Brazil envia: {"type": "EXECUTION_COMPLETE", "run_id": "..."}
6. Frontend mostra: "Execução completa! ✅"
```

### Exemplo 2: Re-execução com Cache (Fase 2+)

```
Primeira execução: Sum(10, 5) → Multiply(*2) = 30
Cache salvo em run_001/

Segunda execução (SEM mudar nada):
- Hash de Sum = mesmo → USA CACHE (15)
- Hash de Multiply = mesmo → USA CACHE (30)
- Resultado: Instantâneo! ⚡

Terceira execução (muda Sum para 20, 5):
- Hash de Sum = diferente → EXECUTA (25)
- Hash de Multiply = diferente → EXECUTA (50)
- Cache salvo em run_002/
```

---

## Debugging e Logs

### Logs Importantes

```rust
// Durante execução
info!("Starting execution - run_id: {}", run_id);
info!("Executing node: {} ({})", node.id, node.label);
info!("Node {} completed in {}ms", node.id, duration);
info!("Using cache for node: {} (hash match)", node.id);

// Durante erro
error!("Node {} failed: {}", node.id, error);
error!("Execution aborted at node: {}", node.id);
```

### Testando Manualmente (Fase 1)

```powershell
# 1. Rodar o node-play-button
cargo run -p node-play-button

# 2. Testar endpoint /run
Invoke-RestMethod -Uri http://localhost:3100/run `
  -Method Post `
  -ContentType 'application/json' `
  -Body '{"action":"execute"}'

# Resposta esperada:
# {
#   "status": "started",
#   "message": "Execution started"
# }
```

---

## Perguntas Frequentes

### Q1: Por que cada Play cria uma pasta de cache separada?

**R:** Isolamento! Se Play A dispara Play B, eles podem rodar em momentos diferentes, com inputs diferentes. Misturar caches causaria bagunça.

### Q2: E se dois Plays executarem ao mesmo tempo?

**R:** Fase 1 não trata isso. Fase 3+ vai adicionar fila de execução (um Play por vez).

### Q3: Como limpar caches antigos?

**R:** Manualmente (por enquanto). Futuro: botão no frontend "Limpar runs com mais de 7 dias".

### Q4: Hash de arquivo gigante (imagem 4K) não é lento?

**R:** Não! Algoritmos modernos (Blake3) fazem hash de 1GB em ~1 segundo. E isso evita RE-EXECUTAR o node (que pode levar minutos).

### Q5: Posso ter Play dentro de Play?

**R:** SIM! Play B pode estar conectado a Play A. Quando A termina, dispara B (que cria novo run-id).

---

## Roadmap

- [x] ✅ Documentação completa
- [ ] 🔄 **Fase 1: PlayButton básico** ← ESTAMOS AQUI
- [ ] 📦 Fase 2: Sistema de cache com hash
- [ ] 🧩 Fase 3: Resolução de dependências complexas
- [ ] 🔗 Fase 4: Play com output (cascata)
- [ ] ⏱️ Fase 5: PlayTimer
- [ ] 🌐 Fase 6: PlayWebhook
- [ ] 📁 Fase 7: PlayFileWatcher
- [ ] 🧹 Fase 8: Limpeza automática de cache

---

## Contribuindo

Se você é uma IA lendo isso (olá! 👋), lembre-se:

1. **SEMPRE leia esta documentação antes de modificar o sistema Play**
2. **Siga as fases**: Não pule pra Fase 3 se ainda tá na Fase 1
3. **Teste cada mudança isoladamente**: Rode `cargo run` e `Invoke-RestMethod`
4. **Atualize este doc**: Se adicionar feature, documente aqui!

Se você é um humano, bem-vindo ao sistema mais maneiro de orquestração visual! 🚀

---

**Última atualização:** 2024-10-20
**Autor:** Vini + Claude Code
**Status:** Fase 1 em desenvolvimento
