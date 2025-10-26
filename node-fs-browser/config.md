Esta seção detalha os campos utilizados na estrutura do config.yaml para
definir um node dentro do ecossistema ndnm.

Campos de Nível Superior node_id_hash (string, Obrigatório):

Descrição: O identificador único e determinístico do node. Garante que
cada definição de node tenha uma identidade única, mesmo que o código
seja recompilado.

Geração: Calculado usando um algoritmo de hash (ex: SHA-256) sobre uma
string que combina um identificador do criador e um nome único do node
(ex: hash("criador" + "nome-do-node")).

Uso: Usado pelo ndnm-brazil, Hermes, frontend (como type interno do
React Flow) e nos arquivos de workspace para referenciar inequivocamente
este tipo de node.

label (string, Obrigatório):

Descrição: O nome amigável do node, exibido na interface do usuário
(UI), como no catálogo de nodes e no próprio corpo do node renderizado.

Exemplo: "📂 Gerenciador de Arquivos (Dinâmico)"

node_type (string, Obrigatório):

Descrição: Uma categoria funcional para o node. Ajuda a agrupar nodes
similares na UI e dá uma dica semântica sobre o propósito do node.
Múltiplos nodes (com node_id_hash diferentes) podem compartilhar o mesmo
node_type.

Exemplo: "filesystem", "math", "latent", "loader", "sampler", "clip"

sections (list, Obrigatório):

Descrição: Define os grupos lógicos de pontos de entrada (inputs) e
saída (outputs) do node. Cada item na lista representa uma "seção"
visual ou funcional de I/O.

input_fields (list, Opcional):

Descrição: Define os controles de interface (widgets) que devem ser
renderizados dentro do corpo do node na UI. Permite que o usuário
configure parâmetros específicos do node diretamente.

Campos Dentro de sections Cada item na lista sections é um objeto que
define um grupo de slots:

section_name (string, Obrigatório):

Descrição: Um identificador interno único para esta seção dentro do
node. Usado pelo sistema para referenciar a seção.

Exemplo: "copy_here", "internal_files"

section_label (string, Opcional):

Descrição: Um título opcional para esta seção, que pode ser usado pela
UI para agrupar visualmente os slots pertencentes a ela.

Exemplo: "Copiar Para Cá", "Arquivos na Pasta"

behavior (string, Obrigatório):

Descrição: Define como os slots dentro desta seção se comportam e são
gerenciados, especialmente em relação à dinamicidade. Valores possíveis:

"fixed": (Implícito se slots ou input/output diretos forem usados, mas
recomendado explicitar) A seção contém um ou mais slots fixos e
nomeados, definidos diretamente. A estrutura da UI não muda.

"auto_increment": Usado principalmente para inputs. Define um template
(slot_template). A UI começa com uma instância do template. Quando uma
conexão é feita, a UI automaticamente cria e exibe uma nova instância do
mesmo template abaixo, permitindo múltiplas entradas sequenciais, cada
uma com sua própria conexão.

"dynamic_per_file": Usado em nodes que interagem com o sistema de
arquivos. Define um template (slot_template). A UI (em conjunto com o
backend) irá dinamicamente criar e remover instâncias desse template
(geralmente pares de input/output) com base nos arquivos encontrados em
um diretório especificado (target_directory nos input_fields).

slot_template (object, Obrigatório se behavior for dinâmico):

Descrição: Define a estrutura (o "molde") para os slots que serão
gerados dinamicamente pelos comportamentos auto_increment ou
dynamic_per_file. Contém as chaves input e/ou output.

input (object, Usado em slot_template ou diretamente na seção se
behavior: fixed e for um único input):

Descrição: Define as propriedades de um slot de entrada.

output (object, Usado em slot_template ou diretamente na seção se
behavior: fixed e for um único output):

Descrição: Define as propriedades de um slot de saída.

slots (list, Usado diretamente na seção se behavior: fixed e houver
múltiplos slots fixos):

Descrição: Uma lista onde cada item define um slot fixo individual
usando as chaves input: {...} ou output: {...}. (Exemplo no KSampler
v9).

Campos Dentro de input / output Estes campos definem as propriedades de
um slot individual:

name (string, Obrigatório):

Descrição: O nome técnico do slot, usado internamente pelo Hermes e como
base para o handleId na UI. Para slots dinâmicos, este pode ser um nome
base que a implementação completa (ex: adicionando \_0, \_1 ou
{filename}).

Exemplo: "copy_input", "copied_output", "internal_input", "sum_output"

label (string, Opcional):

Descrição: O texto exibido ao lado do handle na UI. Pode conter
placeholders como {filename} que a UI substituirá dinamicamente. Se
omitido, a UI pode usar o name.

Exemplo: "Novo Arquivo", "Arquivo Copiado", "{filename}", "Soma"

type (string, Obrigatório):

Descrição: Especifica o tipo de dado esperado (para input) ou fornecido
(para output). Ajuda na validação e pode influenciar a UI. Uma lista
formal de tipos válidos precisa ser mantida (ex: INT, STRING, FLOAT,
BOOLEAN, FILE_PATH, FILE_CONTENT, LIST`<TYPE>`{=html}, LATENT, MODEL,
CLIP_EMBEDDING, ANY, etc.).

Exemplo: "FILE_CONTENT", "LIST`<STRING>`{=html}"

connections (integer \| "n", Obrigatório):

Descrição: Define quantas conexões são permitidas para este slot.

Valores:

1: Exatamente uma conexão permitida (típico para input).

"n": Zero ou múltiplas conexões permitidas (típico para output).

Campos Dentro de input_fields Cada item na lista input_fields define um
controle na UI interna do node:

name (string, Obrigatório):

Descrição: O identificador interno para este campo/controle. Usado para
armazenar/recuperar o valor do controle e referenciado pela lógica do
node.

Exemplo: "target_directory", "refresh_button", "value", "seed"

label (string, Opcional):

Descrição: O texto exibido ao lado ou acima do controle na UI.

Exemplo: "Pasta Gerenciada", "Atualizar Visualização"

type (string, Obrigatório):

Descrição: Define o tipo de widget a ser renderizado na UI. Uma lista
formal de tipos de widget válidos precisa ser mantida pelo frontend (ex:
text, number, boolean, select, button, file_picker, directory_picker,
etc.).

Exemplo: "text", "button", "number", "select"

Outros campos dependentes do type: Widgets como select podem ter campos
adicionais como options e default.
