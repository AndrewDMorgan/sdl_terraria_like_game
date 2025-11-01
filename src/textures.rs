
pub fn get_texture_atlas<const TEXTURE_COUNT: usize>(path: &str, tile_size: (u32, u32)) -> [[u32; 64]; TEXTURE_COUNT] {
    let mut textures = [[0u32; 64]; TEXTURE_COUNT];

    // read through all png files in the directory
    // load each (splicing it by the tile size)
    // for each slice, if it's not empty, add it to the textures array

    textures
}
