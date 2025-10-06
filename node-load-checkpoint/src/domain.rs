// node-load-checkpoint/src/domain.rs
use crate::{AppError, ModelType, Output}; // Adicionado ModelType
use safetensors::SafeTensors;
use std::fs;
use std::path::Path;

pub fn load_and_analyze_checkpoint(path: &Path) -> Result<Output, AppError> {
    let buffer = fs::read(path)
        .map_err(|e| AppError::bad(format!("não foi possível ler o arquivo {:?}: {}", path, e)))?;

    let tensors = SafeTensors::deserialize(&buffer)
        .map_err(|e| AppError::bad(format!("falha ao analisar o arquivo safetensors: {}", e)))?;

    // Coleta todas as chaves para análise
    let keys: Vec<_> = tensors.iter().map(|(key, _)| key).collect();

    // --- Lógica de Detetive ---

    // 1. É uma Inversão Textual?
    if keys.contains(&"string_to_param") {
        return Ok(Output {
            file_path: path.to_string_lossy().into_owned(),
            model_type: ModelType::TextualInversion,
            tensor_count: keys.len(),
            ..Default::default() // Preenche o resto com valores padrão
        });
    }

    // 2. É um LoRA?
    let lora_keys_count = keys.iter().filter(|k| k.contains(".lora_")).count();
    // Se mais da metade das chaves forem de LoRA, é uma aposta segura.
    if lora_keys_count > keys.len() / 2 {
        return Ok(Output {
            file_path: path.to_string_lossy().into_owned(),
            model_type: ModelType::Lora,
            tensor_count: keys.len(),
            ..Default::default()
        });
    }

    // 3. Se não, é um Checkpoint Completo.
    let mut model_keys = Vec::new();
    let mut clip_keys = Vec::new();
    let mut vae_keys = Vec::new();

    for key in keys {
        if key.starts_with("model.diffusion_model.") {
            model_keys.push(key);
        } else if key.starts_with("cond_stage_model.") || key.starts_with("conditioner.embedders.") {
            clip_keys.push(key);
        } else if key.starts_with("first_stage_model.") {
            vae_keys.push(key);
        }
    }

    Ok(Output {
        file_path: path.to_string_lossy().into_owned(),
        model_type: ModelType::Checkpoint,
        tensor_count: model_keys.len() + clip_keys.len() + vae_keys.len(),
        model_tensor_count: model_keys.len(),
        clip_tensor_count: clip_keys.len(),
        vae_tensor_count: vae_keys.len(),
        model_keys_preview: model_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
        clip_keys_preview: clip_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
        vae_keys_preview: vae_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
    })
}