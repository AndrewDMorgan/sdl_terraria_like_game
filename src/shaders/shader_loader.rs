use metal::Device;

use crate::{logging::logging::{Log, LoggingError, Logs}, shaders::shader_handler::Shader, textures::textures::{get_font_atlas, get_texture_atlas}};

pub static MAX_ENTITIES: usize = 1024;
static MAX_PARTICLES: usize = 2048;
static MAX_TEXTS: usize = 1024;

const TEXTURE_COUNT: usize = u16::MAX as usize * 4;

static TILE_SIZE: (u32, u32) = (8, 8);
static MAX_CHARACTERS: usize = 32;

/// Loads all shaders required for the game and returns them as an array
pub fn load_game_shaders(device: &Device, max_screen_size: (u32, u32), logs: &mut Logs) -> Result<Shader, String> {
    // this does have to stay up to date with the number of shaders, but
    // since all the shaders do have to be loaded, it should be fine to assume the number of shaders
    Ok(
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
                (size_of::<bool>() * 64 * u8::MAX as usize) as u64,
                size_of::<u32>() as u64, // default_font_size
                (size_of::<u8>() as u32 * max_screen_size.0 * max_screen_size.1 * 3) as u64,
            ], "ComputeShader")?;
            
            // loading the textures
            let mut total_textures_loaded_entities = 0;
            shader.update_buffer_slice(3, 
                &get_texture_atlas::<TEXTURE_COUNT, 64>(
                    "textures/entities/" , TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT], &mut total_textures_loaded_entities
                )?
            )?; // entity_textures
            let mut total_textures_loaded_tiles = 0;
            shader.update_buffer_slice(4, 
                &get_texture_atlas::<TEXTURE_COUNT, 64>(
                    "textures/tiles/"    , TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT], &mut total_textures_loaded_tiles
                )?
            )?; // tile_textures
            let mut total_textures_loaded_particles = 0;
            shader.update_buffer_slice(5, 
                &get_texture_atlas::<TEXTURE_COUNT, 64>(
                    "textures/particles/", TILE_SIZE, vec![[0u32; (TILE_SIZE.0 * TILE_SIZE.1) as usize]; TEXTURE_COUNT], &mut total_textures_loaded_particles
                )?
            )?; // particle_textures

            // the font size is 8 in this case
            let font_atlas = get_font_atlas::<8, 64>("textures/fonts/default_font.png")?;
            shader.update_buffer_slice(16, &font_atlas)?;
            shader.update_buffer(17, 8)?;

            logs.push(Log { message: format!(
                "Loaded {} entity textures, {} tile textures, and {} particle textures into the GPU.", 
                total_textures_loaded_entities - 1, // minus 1 to account for the empty texture
                total_textures_loaded_tiles - 1,
                total_textures_loaded_particles - 1
            ), level: LoggingError::Info }, 21, crate::logging::logging::LogType::Information);

            shader
        }
    )
}

