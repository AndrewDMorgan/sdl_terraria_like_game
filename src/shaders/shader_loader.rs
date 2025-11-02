use metal::Device;

use crate::{shaders::shader_handler::Shader, textures::textures::get_texture_atlas};

static MAX_ENTITIES: usize = 1024;
static MAX_PARTICLES: usize = 2048;
static MAX_TEXTS: usize = 1024;

const TEXTURE_COUNT: usize = u16::MAX as usize * 4;

static TILE_SIZE: (u32, u32) = (8, 8);
static MAX_CHARACTERS: usize = 32;

/// Loads all shaders required for the game and returns them as an array
pub fn load_game_shaders(device: &Device, max_screen_size: (u32, u32)) -> Result<[Shader; 1], String> {
    // this does have to stay up to date with the number of shaders, but
    // since all the shaders do have to be loaded, it should be fine to assume the number of shaders
    Ok([
        {
            // wow, having the wrapper handle everything not only cleaned it up, but somehow I haven't gotten a single segfault yet
            let mut shader = Shader::new(&device, "shaders/shader.metal", &[
                size_of::<u32>() as u64, // pitch
                size_of::<u32>() as u64, // width
                size_of::<u32>() as u64, // height
                (size_of::<u32>() * TILE_SIZE.0 as usize * TILE_SIZE.1 as usize * TEXTURE_COUNT) as u64, // entity_textures
                (size_of::<u32>() * TILE_SIZE.0 as usize * TILE_SIZE.1 as usize * TEXTURE_COUNT) as u64, // tile_textures
                (size_of::<u32>() * TILE_SIZE.0 as usize * TILE_SIZE.1 as usize * TEXTURE_COUNT) as u64, // particle_textures
                size_of::<u32>() as u64, // tile_map_width
                size_of::<u32>() as u64, // tile_map_height
                (size_of::<u64>() * 1024 * 1024 * 4) as u64, // tile_map (assuming a maximum width, height, and there will always be 4 layers)
                (size_of::<f32>() * 4) as u64, // camera_position + rotation + scale
                size_of::<u32>() as u64, // num_entities
                (size_of::<u64>() * 2 * MAX_ENTITIES) as u64, // max of 1024 entities on screen at a given time
                size_of::<u32>() as u64, // num_particles
                (size_of::<u64>() * 2 * MAX_PARTICLES) as u64, // max of 2048 particles on screen at a given time
                size_of::<u64>() as u64, // num_texts
                ((size_of::<u64>() * 2 + size_of::<u8>() * MAX_CHARACTERS) * MAX_TEXTS) as u64, // max of 1024 text entries on screen at a given time
                (size_of::<u8>() as u32 * max_screen_size.0 * max_screen_size.1 * 3) as u64,
            ], "ComputeShader")?;
            
            // loading the textures
            shader.update_buffer_slice(3, 
                &get_texture_atlas::<TEXTURE_COUNT>(
                    "textures/entities/" , TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT]
                )?
            )?; // entity_textures
            shader.update_buffer_slice(4, 
                &get_texture_atlas::<TEXTURE_COUNT>(
                    "textures/tiles/"    , TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT]
                )?
            )?; // tile_textures
            shader.update_buffer_slice(5, 
                &get_texture_atlas::<TEXTURE_COUNT>(
                    "textures/particles/", TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT]
                )?
            )?; // particle_textures
            shader
        }
    ])
}

