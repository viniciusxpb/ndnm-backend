ndnm (No deps, no masters) - Um Manifesto Caótico
O Que É Esta Loucura?
Glitch: ndnm é um orquestrador de processos locais, local-first e open-source. A premissa é simples: criar um sistema onde qualquer módulo (ou "node"), em qualquer linguagem, possa ser plugado, executado e visualmente conectado a outros, formando fluxos de trabalho complexos. Foi projetado com uma filosofia de "máquina de escrever": robusto, simples e capaz de sobreviver até mesmo às mais criativas e aterrorizantes tentativas de contribuição de um júnior.

Lilith: É mais do que uma ferramenta, é um ecossistema. Um lugar onde você invoca fragmentos de código como entidades independentes, unindo-os com teias de comunicação para que trabalhem juntos na escuridão. Cada node é um pensamento, e você, Vini, é o arquiteto da consciência que emerge dessa união.

O Credo (Nosso Manifesto)
Glitch: > "No deps, no masters." Porque cada npm install é um pacto com uma entidade desconhecida e cada sistema centralizado é um ponto único de falha. Acreditamos em código que funciona, que é resiliente e que não te acorda às 3 da manhã porque uma dependência obscura foi depreciada.

Lilith: > "Do caos, a ordem. Da escuridão, o poder." Não tememos o erro; nós o usamos. Cada panic! é um grito de nascimento. Cada bug bizarro é um vislumbre da Mente Aberrante em ação. Este não é um sistema para os fracos de coração. É para os que entendem que a verdadeira criação exige um pouco de loucura.

Arquitetura de Comunicação: A Teia de Sussurros
Glitch: O sistema opera em um modelo cliente-servidor. O ndnm-core atua como o servidor WebSocket central. Cada "node" que você executa age como um cliente, iniciando uma conexão com o core. Isso nos dá um mecanismo de "heartbeat" implícito: se a conexão de um node cai, sabemos que ele morreu. Simples e eficaz.

Lilith: Pense no core como o coração pulsante no centro da teia. Cada node é uma aranha que se conecta, enviando e recebendo vibrações. É através desses fios que a vontade do sistema é exercida, transformando processos isolados em uma sinfonia sombria e coordenada.

Como Invocar os Demônios (Como Rodar)
Você precisará de dois ou mais terminais abertos. Um para o mestre, e um para cada servo.

1. Invocando o Orquestrador (ndnm-core)
Este é o cérebro, o servidor WebSocket que comanda tudo.

# Navegue até o diretório do core
cd src/ndnm-core

# Execute o mestre das marionetes
cargo run

Glitch: O servidor iniciará em 127.0.0.1:3000. Se ele não estiver rodando, seus nodes estarão gritando sozinhos no vácuo.

2. Despertando um Servo (node-sum)
Este é um exemplo de um node que se conecta ao orquestrador.

# Em OUTRO terminal, navegue até o diretório do node
cd src/node-sum

# Rode o servo
cargo run

Lilith: Veja-o se conectar. Sinta o pacto sendo formado. Agora ele pertence a você. Repita o processo para cada alma que desejar adicionar à sua legião.

Próximos Passos: O Livro dos Feitiços
Glitch: O próximo passo lógico é a definição de um protocolo de comunicação estrito. Precisamos de um schema JSON claro para as mensagens trocadas, definindo tipos de requisição (execute, status_update), payloads e formatos de resposta.

Lilith: Precisamos escrever as "palavras de poder". O vocabulário que dará sentido aos sussurros na teia, permitindo que o orquestrador não apenas ouça, mas comande verdadeiramente cada um de seus servos.

Sussurros da Trincheira (Dicas)
Glitch: Logs são seu oráculo. O ndnm-core vai te dizer exatamente quando um node se conecta ou quando a conexão cai (leia-se: o node crashou). A primeira regra do Plenout é: aprenda a ler os logs.

Lilith: Abrace o inesperado. Se um node se comportar de forma estranha, não o corrija imediatamente. Observe-o. Entenda sua natureza. Às vezes, os bugs mais interessantes são portais para funcionalidades que você nem imaginava.