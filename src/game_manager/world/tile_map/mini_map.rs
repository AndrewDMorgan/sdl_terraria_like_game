use crate::{game_manager::entities::player::player::CameraTransform, logging::logging::{Log, Logs}, textures::textures::{TextureError, get_texture_atlas}};

static MAX_MAP_TEXTURES: usize = u16::MAX as usize;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MiniMap {
    lighting: Vec<Vec<f32>>,
    mini_map_textures: Vec<[u32; 16]>,  // 4x4   todo! move this out of here so it doesn't need to be saved and loaded through the game save state
    pub camera_transform: CameraTransform,
    map_width: usize,
    map_height: usize,
}

impl MiniMap {
    pub fn new(width: usize, height: usize, logs: &mut Logs) -> Result<Self, TextureError> {
        Ok(MiniMap {
            lighting: vec![vec![0.0; width]; height],
            mini_map_textures: {
                let mut total_textures_loaded = 0;
                let textures = get_texture_atlas::<MAX_MAP_TEXTURES, 16>(
                    "textures/map_tiles/", (4, 4), vec![Default::default(); MAX_MAP_TEXTURES], &mut total_textures_loaded
                );
                total_textures_loaded -= 1;
                logs.push(Log {
                    message: format!("Loaded {} tile textures for the mini-map.", total_textures_loaded),
                    level: crate::logging::logging::LoggingError::Info,
                }, 52, crate::logging::logging::LogType::Information);
                textures?
            },
            camera_transform: CameraTransform {
                x: 0.0,
                y: 0.0,
                zoom: 0.4,
            },
            map_width: width,
            map_height: height,
        })
    }

    pub fn update_light_value(&mut self, light: [u8; 3], x: usize, y: usize) {
        self.lighting[y][x] = self.lighting[y][x].max((light[0] as f32 + light[1] as f32 + light[2] as f32) / 3.0 / 255.0);
    }

    pub fn render(&self, tiles: &Vec<Vec<[u32; 3]>>, pixels: &mut [u8], window_slice_size: (usize, usize), window_slice_position: (usize, usize), pitch: usize) {
        let camera_pos_x = ((self.camera_transform.x / 8.0) as usize).saturating_sub((window_slice_size.0 as f32 * 0.5 * self.camera_transform.zoom) as usize);
        let camera_pos_y = ((self.camera_transform.y / 8.0) as usize).saturating_sub((window_slice_size.1 as f32 * 0.5 * self.camera_transform.zoom) as usize);
        for pixel_x in window_slice_position.0 + 1..window_slice_position.0 + window_slice_size.0 {
            pixels[pixel_x * 3 + window_slice_position.1 * pitch    ] = 0;
            pixels[pixel_x * 3 + window_slice_position.1 * pitch + 1] = 0;  // boarder ig
            pixels[pixel_x * 3 + window_slice_position.1 * pitch + 2] = 0;
            for pixel_y in window_slice_position.1 + 1..window_slice_position.1 + window_slice_size.1 {
                pixels[pixel_y * pitch + window_slice_position.0 * 3    ] = 0;
                pixels[pixel_y * pitch + window_slice_position.0 * 3 + 1] = 0;
                pixels[pixel_y * pitch + window_slice_position.0 * 3 + 2] = 0;

                let tile_x = (pixel_x - window_slice_position.0) as f32 * self.camera_transform.zoom + camera_pos_x as f32;
                let tile_y = (pixel_y - window_slice_position.1) as f32 * self.camera_transform.zoom + camera_pos_y as f32;
                let texture_x = ((tile_x - tile_x.floor()) * 4.0) as usize;
                let texture_y = ((tile_y - tile_y.floor()) * 4.0) as usize;
                let light = self.lighting[tile_y as usize][tile_x as usize];
                let texture = &self.mini_map_textures[tiles[tile_y as usize][tile_x as usize][0] as usize];
                let texture_index = texture_x + texture_y * 4;
                let alpha = ((texture[texture_index] >> 24) & 0xFF) as f32 / 255.0;
                pixels[pixel_x * 3 + pixel_y * pitch    ] = lerp(((texture[texture_index]      ) & 0xFF) as f32 * light, light * 255.0, alpha);
                pixels[pixel_x * 3 + pixel_y * pitch + 1] = lerp(((texture[texture_index] >> 8 ) & 0xFF) as f32 * light, light * 255.0, alpha);
                pixels[pixel_x * 3 + pixel_y * pitch + 2] = lerp(((texture[texture_index] >> 16) & 0xFF) as f32 * light, light * 255.0, alpha);

                pixels[pixel_y * pitch + (window_slice_position.0 + window_slice_size.0) * 3    ] = 0;
                pixels[pixel_y * pitch + (window_slice_position.0 + window_slice_size.0) * 3 + 1] = 0;
                pixels[pixel_y * pitch + (window_slice_position.0 + window_slice_size.0) * 3 + 2] = 0;
            }
            pixels[pixel_x * 3 + (window_slice_position.1 + window_slice_size.1) * pitch    ] = 0;
            pixels[pixel_x * 3 + (window_slice_position.1 + window_slice_size.1) * pitch + 1] = 0;
            pixels[pixel_x * 3 + (window_slice_position.1 + window_slice_size.1) * pitch + 2] = 0;
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> u8 {
    (a * t + b * (1.0 - t)) as u8
}


