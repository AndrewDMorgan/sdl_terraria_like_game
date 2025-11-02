
// a basic error type to make error handling slightly cleaner
#[derive(Debug)]
pub struct TextureError {
    details: String,
}

impl From<TextureError> for String {
    fn from(details: TextureError) -> String {
        format!("[Texture Error] {:?}", details.details)
    }
}

pub fn get_texture_atlas<const TEXTURE_COUNT: usize>(path: &str, tile_size: (u32, u32), mut textures: Vec<[u32; 64]>) -> Result<Vec<[u32; 64]>, TextureError> {
    // read through all png files in the directory
    // load each (splicing it by the tile size)
    // for each slice, if it's not empty, add it to the textures array
    let entries = std::fs::read_dir(path).map_err(|e| TextureError { details: format!("Failed to read texture directory: {}", e) })?;
    let mut texture_index = 1 ; // reserving 0 for empty texture
    for entry in entries {
        let entry = entry.map_err(|e| TextureError { details: format!("Failed to read texture directory entry: {}", e) })?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            let img = image::open(&path)
                .map_err(|e| TextureError { details: format!("Failed to open texture image '{}': {}", path.display(), e) })?
                .to_rgba8();
            let (img_width, img_height) = img.dimensions();
            let tiles_x = img_width / tile_size.0;
            let tiles_y = img_height / tile_size.1;
            for ty in 0..tiles_y {
                for tx in 0..tiles_x {
                    if texture_index >= TEXTURE_COUNT {
                        return Err(TextureError { details: format!(
                            "Texture atlas overflow: more than {} textures found in '{}'", TEXTURE_COUNT, path.display()
                        ) });
                    }
                    let mut sum = 0u64;
                    for px in 0..tile_size.0 {
                        for py in 0..tile_size.1 {
                            let pixel = img.get_pixel(tx * tile_size.0 + px, ty * tile_size.1 + py);
                            let r = pixel[0] as u32;
                            let g = pixel[1] as u32;
                            let b = pixel[2] as u32;
                            let a = pixel[3] as u32;
                            sum += r as u64 + g as u64 + b as u64 + a as u64;
                            textures[texture_index][(py * tile_size.0 + px) as usize] =
                                (a << 24) | (b << 16) | (g << 8) | r;
                        }
                    }
                    // if the tile is blank, ignor it
                    if sum > 0 {  texture_index += 1;  }
                }
            }
        }
    } Ok(textures)
}
