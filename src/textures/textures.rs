
// a basic error type to make error handling slightly cleaner
#[derive(Debug)]
pub struct TextureError {
    pub details: String,
}

impl From<TextureError> for String {
    fn from(details: TextureError) -> String {
        format!("[Texture Error] {:?}", details.details)
    }
}

// the vector passed in should have the maximum texture count already allocated before passing it in or else this may panic at runtime
pub fn get_texture_atlas<const TEXTURE_COUNT: usize, const RESULT_SIZE: usize>(path: &str, tile_size: (u32, u32), mut textures: Vec<[u32; RESULT_SIZE]>, total_textures_loaded: &mut usize) -> Result<Vec<[u32; RESULT_SIZE]>, TextureError> {
    // read through all png files in the directory
    // load each (splicing it by the tile size)
    // for each slice, if it's not empty, add it to the textures array
    let entries = std::fs::read_dir(path).map_err(|e| TextureError { details: format!("Failed to read texture directory for {}: {}", path, e) })?;
    let mut texture_index = 1 ; // reserving 0 for empty texture
    
    // sorting so they are pulled in by a consistent order (before it was a pain to align them; adding a new texture would shift everything which isn't maintainable)
    // now, all texture names can begin with a number or identifier which will be used to sort it instead so it's consistent
    let mut entries = entries.collect::<Vec<_>>();
    entries.sort_by_key(|e| e.as_ref().map(|e| e.path()).unwrap_or_default());
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
    }
    *total_textures_loaded = texture_index;
    Ok(textures)
}

pub fn get_font_atlas<const FONT_SIZE: u32, const FONT_SIZE_SQUARE: usize>(path: &str) -> Result<Vec<[bool; FONT_SIZE_SQUARE]>, TextureError> {
    let mut atlas = vec![];
    let img = image::open(path)
        .map_err(|e| TextureError { details: format!("Failed to open texture image '{}': {}", path, e) })?
        .to_rgba8();
    let (img_width, _img_height) = img.dimensions();
    for texture in 0..img_width / FONT_SIZE {
        let mut char_data = [false; FONT_SIZE_SQUARE];
        for y in 0..FONT_SIZE {
            for x in 0..FONT_SIZE {
                let pixel = img.get_pixel(texture * FONT_SIZE as u32 + x, y);
                char_data[(y * FONT_SIZE + x) as usize] = pixel[0] > 0 || pixel[1] > 0 || pixel[2] > 0 || pixel[3] > 0;
            }
        }
        atlas.push(char_data);
    }
    Ok(atlas)
}

