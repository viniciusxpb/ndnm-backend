# C:/Projetos/ndnm/ndnm-backend/node-clip-text-encode-py/main.py

from fastapi import FastAPI
from pydantic import BaseModel
from transformers import CLIPProcessor, CLIPModel
import torch

# --- Carregando o Modelo (Acontece uma vez quando o servidor inicia) ---
# A gente já deixa o modelo e o processador prontos na memória.
# Na primeira vez que rodar, ele vai baixar o modelo (pode demorar um pouco!)
print("Carregando o modelo CLIP... Isso pode levar alguns minutos na primeira vez.")
modelo = CLIPModel.from_pretrained("openai/clip-vit-base-patch32")
processador = CLIPProcessor.from_pretrained("openai/clip-vit-base-patch32")
print("Modelo carregado com sucesso!")
# --------------------------------------------------------------------

# Define a estrutura do JSON que a gente vai receber
class Input(BaseModel):
    text: str

# Cria a nossa aplicação
app = FastAPI()

# A rota de saúde, pra ver se o node tá vivo
@app.get("/health")
def health():
    return {"status": "ok"}

# A rota principal que faz a mágica acontecer
@app.post("/run")
def run(input_data: Input):
    print(f"Recebido texto para encodar: '{input_data.text}'")

    # Usa o processador para transformar o texto em "tokens" que o modelo entende
    inputs = processador(text=input_data.text, return_tensors="pt", padding=True)

    # Pede para o modelo gerar os "embeddings" (a representação numérica do texto)
    with torch.no_grad(): # Desliga o cálculo de gradientes pra economizar memória e acelerar
        text_features = modelo.get_text_features(**inputs)

    # O resultado é um Tensor. A gente converte pra uma lista de Python pra poder enviar como JSON.
    embedding = text_features[0].tolist()

    print(f"Embedding gerado com sucesso! Tamanho: {len(embedding)}")

    return {
        "status": "success",
        "text": input_data.text,
        "embedding_shape": list(text_features.shape),
        "embedding_preview": embedding[:5] # Mostra só os 5 primeiros números pra não poluir a tela
    }

# Comando para rodar o servidor (se a gente executar `python main.py`)
if __name__ == "__main__":
    import uvicorn
    # A porta 3007 será nosso novo lar!
    uvicorn.run(app, host="0.0.0.0", port=3007)