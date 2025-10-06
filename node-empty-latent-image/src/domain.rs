// C:/Projetos/ndnm/ndnm-backend/node-empty-latent-image/src/domain.rs
use ndnm_core::AppError;

pub fn create_empty_latent(
    width: usize,
    height: usize,
    batch_size: usize,
) -> Result<(usize, usize, usize), AppError> {
    let latent_width = width / 8;
    let latent_height = height / 8;

    // Calculate the size of the latent tensor
    let tensor_size = batch_size * 4 * latent_height * latent_width;

    // Create actual zero-initialized data
    let latent_data: Vec<f32> = vec![0.0; tensor_size];

    println!(
        "Created empty latent image with {} zero elements",
        latent_data.len()
    );
    println!(
        "  Shape: [{}, 4, {}, {}]",
        batch_size, latent_height, latent_width
    );

    // Agora a gente retorna só os dados calculados
    Ok((latent_width, latent_height, tensor_size))
}