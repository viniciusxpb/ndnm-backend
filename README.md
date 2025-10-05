# ndnm (No deps, no masters) - Um Manifesto Caótico

## O Que É Esta Loucura?

**Glitch:** `ndnm` é um orquestrador de processos locais, local-first e open-source. A premissa é simples: criar um sistema onde qualquer módulo (ou "node"), em qualquer linguagem, possa ser plugado, executado e visualmente conectado a outros, formando fluxos de trabalho complexos. Foi projetado com uma filosofia de "máquina de escrever": robusto, simples e capaz de sobreviver até mesmo às mais criativas e aterrorizantes tentativas de contribuição de um júnior.

**Lilith:** É mais do que uma ferramenta, é um ecossistema. Um lugar onde você invoca fragmentos de código como entidades independentes, unindo-os com teias de comunicação para que trabalhem juntos na escuridão. Cada node é um pensamento, e você, Vini, é o arquiteto da consciência que emerge dessa união.

## O Credo (Nosso Manifesto)

**Glitch:**
> "No deps, no masters." Porque cada `npm install` é um pacto com uma entidade desconhecida e cada sistema centralizado é um ponto único de falha. Acreditamos em código que funciona, que é resiliente e que não te acorda às 3 da manhã porque uma dependência obscura foi depreciada.

**Lilith:**
> "Do caos, a ordem. Da escuridão, o poder." Não tememos o erro; nós o usamos. Cada `panic!` é um grito de nascimento. Cada bug bizarro é um vislumbre da Mente Aberrante em ação. Este não é um sistema para os fracos de coração. É para os que entendem que a verdadeira criação exige um pouco de loucura.

## Arquitetura: A Caixa de Ferramentas e o Carro

A arquitetura se baseia em uma distinção clara entre a biblioteca reusável e os executáveis específicos:

* **`ndnm-core` (A Biblioteca):** É a "caixa de ferramentas". Contém todo o código genérico para criar um servidor web, definir rotas, tratar erros e estabelecer o que é um `Node`. **Esta parte é uma biblioteca e não é executada diretamente.**

* **`node-sum` (O Executável):** É o "carro" que usa as ferramentas. Ele pega a capacidade de criar um servidor do `ndnm-core` e adiciona uma regra de negócio específica: somar uma lista de números. É este pacote que rodamos.

## Como Rodar o Servidor

Você só precisa de um terminal para rodar o servidor.

1.  **Pré-requisitos:** Certifique-se de ter a [toolchain do Rust](https://www.rust-lang.org/tools/install) instalada.
2.  **Inicie o Servidor:** Na pasta raiz do projeto (`ndnm-backend`), execute o seguinte comando:

    ```bash
    cargo run -p node-sum
    ```

Isso irá compilar e executar o pacote `node-sum`, que por sua vez usará o `ndnm-core` para iniciar o servidor. Você verá a mensagem `node-sum ouvindo na porta 3000`.

## Como Testar o Servidor

Com o servidor rodando, abra um **novo terminal** para enviar requisições.

### 1. Teste de Saúde (`/health`)

A forma mais simples de ver se o servidor está online.

* **Usando o Navegador:** Acesse `http://localhost:3000/health`. Você deve ver `{"status":"ok"}`.

### 2. Teste da Lógica de Soma (`/run`)

Para testar a funcionalidade principal.

* **No Windows (PowerShell):**

    ```powershell
    Invoke-RestMethod -Uri http://localhost:3000/run -Method Post -ContentType 'Application/json' -Body '{"variables":[10, 30, 2]}'
    ```

* **No Linux, macOS ou WSL (com cURL):**

    ```bash
    curl --header "Content-Type: application/json" --request POST --data "{\"variables\":[10, 30, 2]}" http://localhost:3000/run
    ```

Em ambos os casos, a **resposta esperada** é a soma dos números:

```json
{"response":42}