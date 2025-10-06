// node-load-checkpoint/src/domain.rs
use crate::{AppError, Output};
use safetensors::SafeTensors;
use std::fs;
use std::path::Path;

pub fn load_and_analyze_checkpoint(path: &Path) -> Result<Output, AppError> {
    let buffer = fs::read(path)
        .map_err(|e| AppError::bad(format!("não foi possível ler o arquivo {:?}: {}", path, e)))?;

    let tensors = SafeTensors::deserialize(&buffer)
        .map_err(|e| AppError::bad(format!("falha ao analisar o arquivo safetensors: {}", e)))?;

    let mut model_keys = Vec::new();
    let mut clip_keys = Vec::new();
    let mut vae_keys = Vec::new();

    for (key, _view) in tensors.iter() {
        if key.starts_with("model.diffusion_model.") {
            model_keys.push(key);
        // Corrigido: Adicionamos uma verificação para outros prefixos comuns do CLIP (SDXL).
        } else if key.starts_with("cond_stage_model.") || key.starts_with("conditioner.embedders.") {
            clip_keys.push(key);
        } else if key.starts_with("first_stage_model.") {
            vae_keys.push(key);
        }
    }

    Ok(Output {
        file_path: path.to_string_lossy().into_owned(),
        model_tensor_count: model_keys.len(),
        clip_tensor_count: clip_keys.len(),
        vae_tensor_count: vae_keys.len(),
        model_keys_preview: model_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
        clip_keys_preview: clip_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
        vae_keys_preview: vae_keys.into_iter().take(5).map(|s| s.to_string()).collect(),
    })
}