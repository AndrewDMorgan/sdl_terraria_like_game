use crate::game_manager::entities::player::player::CameraTransform;
use crate::game_manager::game::GameError;
use crate::game_manager::world::tile_map::mini_map::MiniMap;
use crate::game_manager::world::world_gen::WorldGenerator;
use crate::logging::logging::{LoggingError, Logs};

pub mod mini_map;

pub static GRASS_IDS: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 47];
pub static DIRT_IDS: &[u32] = &[15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 46];
pub static STONE_IDS: &[u32] = &[30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45];
pub static TILE_LIGHTS: &[(u32, [u8; 3])] = &[(88, [255, 255, 128])];

pub static SOLID_TILES: &[&[u32]] = &[
    GRASS_IDS,
    DIRT_IDS,
    STONE_IDS,
];

#[derive(bincode::Encode, bincode::Decode)]
pub struct EntityLight {
    pub(crate) position: (f32, f32),
    pub(crate) color: (u8, u8, u8, f32),
}

impl EntityLight {
    pub fn new(position: (f32, f32), color: (u8, u8, u8, f32)) -> Self {
        Self { position, color }
    }
}

pub struct TileMapError {
    pub(crate) message: String,
    pub(crate) level: LoggingError,
}

impl From<TileMapError> for GameError {
    fn from(error: TileMapError) -> Self {
        GameError {
            message: error.message,
            severity: match error.level {
                LoggingError::Info => crate::game_manager::game::Severity::Low,
                LoggingError::Warning => crate::game_manager::game::Severity::Medium,
                LoggingError::Error => crate::game_manager::game::Severity::Fatal,
            },
        }
    }
}

impl From<TileMapError> for String {
    fn from(error: TileMapError) -> Self {
        format!("[Tilemap Error of Severity: {:?}] {}", error.level, error.message)
    }
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct TileMap {
    pub tiles: Vec<Vec<[u32; 3]>>,
    lighting: Vec<Vec<[u8; 3]>>,
    pub sky_light: Vec<u32>,
    pub(crate) entity_lights: Vec<(String, EntityLight)>,
    pub(crate) mini_map: mini_map::MiniMap,
}

impl TileMap {
    pub fn new(width: usize, height: usize, world_generator: Option<&WorldGenerator>, logs: &mut Logs) -> Result<Self, TileMapError> {
        let mut tile_map = TileMap {
            tiles: vec![vec![[0; 3]; width]; height],
            lighting: vec![vec![[0; 3]; width]; height],
            sky_light: vec![height as u32; width],
            entity_lights: Vec::new(),
            mini_map: MiniMap::new(width, height, logs).map_err(|e| TileMapError {
                message: format!("Failed to create MiniMap: {:?}", e),
                level: LoggingError::Error,
            })?,
        };
        if let Some(generator) = world_generator {
            generator.generate_tile_map(&mut tile_map)?;
        }
        Ok(tile_map)
    }

    pub fn add_entity_light(&mut self, name: String, position: (f32, f32), color: (u8, u8, u8, f32)) {
        self.entity_lights.push((name, EntityLight::new(position, color)));
    }

    pub fn update_entity_light(&mut self, name: &str, position: (f32, f32), color: (u8, u8, u8, f32)) {
        if let Some(light) = self.entity_lights.iter_mut().find(|(n, _)| n == name) {
            light.1 = EntityLight::new(position, color);
        }
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize, layer: usize) -> &mut u32 {
        &mut self.tiles[y][x][layer]
    }

    pub fn check_aabb_collision(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        let start_x = (x / 8.0).floor() as isize;
        let start_y = (y / 8.0).floor() as isize;
        let end_x = ((x + width) / 8.0).ceil() as isize;
        let end_y = ((y + height) / 8.0).ceil() as isize;

        for tile_y in start_y..end_y {
            for tile_x in start_x..end_x {
                if tile_x < 0 || tile_y < 0 || tile_y as usize >= self.get_map_height() || tile_x as usize >= self.get_map_width() {
                    continue;
                }
                let tile_id = self.get_tile(tile_x as usize, tile_y as usize, 0);
                for solid_ids in SOLID_TILES {
                    if solid_ids.contains(&tile_id) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get_tile(&self, x: usize, y: usize, layer: usize) -> u32 {
        if y >= self.get_map_height() || x >= self.get_map_width() { return 0; }
        self.tiles[y][x][layer]
    }

    pub fn get_map_width(&self) -> usize {
        self.tiles[0].len()
    }

    pub fn get_map_height(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_light_mut(&mut self, x: usize, y: usize) -> &mut [u8; 3] {
        &mut self.lighting[y][x]
    }

    pub fn change_tile(&mut self, tile_x: usize, tile_y: usize, layer: usize, new_tile: u32) -> Result<(), TileMapError> {
        let mut min_edit_height = tile_y;
        let mut max_edit_height = tile_y;
        let mut edit_width = 0;

        let was_removed_light = TILE_LIGHTS.iter().any(|(tile_id, _)| *tile_id == self.get_tile(tile_x, tile_y as usize, 0));

        let mut light_was_edited = false;
        *self.get_tile_mut(tile_x, tile_y, layer) = new_tile;
        if TILE_LIGHTS.iter().any(|(tile_id, _)| *tile_id == self.get_tile(tile_x, tile_y as usize, 0)) {
            *self.get_light_mut(tile_x, tile_y as usize) = TILE_LIGHTS
                .iter()
                .find(|(tile_id, _)| *tile_id == self
                    .get_tile(tile_x, tile_y as usize, 0))
                .ok_or_else(|| TileMapError {
                    message: format!("[Tilemap Error] Unable to locate light for tile at ({}, {}) while updating tiles", tile_x, tile_y),
                    level: LoggingError::Error
                })?.1;
            light_was_edited = true;
        } else if was_removed_light {
            min_edit_height -= 8;
            max_edit_height += 8;
            edit_width = 8;
            for x_index in tile_x.saturating_sub(16)..(tile_x + 16).min(self.get_map_width() - 1) {
                for y_index in (tile_y - 16)..(tile_y + 16).min(self.get_map_height()) {
                    if !TILE_LIGHTS.iter().any(|(tile_id, _)| *tile_id == self.get_tile(x_index, y_index as usize, 0)) {
                        *self.get_light_mut(x_index, y_index as usize) = [0, 0, 0];
                        continue;
                    }
                    *self.get_light_mut(x_index, y_index as usize) = TILE_LIGHTS
                        .iter()
                        .find(|(tile_id, _)| *tile_id == self
                            .get_tile(x_index, y_index as usize, 0))
                        .ok_or_else(|| TileMapError {
                            message: format!("[Tilemap Error] Unable to locate light for tile at ({}, {}) while updating tiles and removing light source", x_index, y_index),
                            level: LoggingError::Error
                        })?.1;
                }
            }
            light_was_edited = true;
        }

        let is_solid = SOLID_TILES.iter().any(|solid_ids| solid_ids.contains(&new_tile));
        if tile_y <= self.sky_light[tile_x] as usize && is_solid {
            self.sky_light[tile_x] = tile_y as u32;
        } else if !is_solid {
            self.sky_light[tile_x] = self.tiles.iter().enumerate().find_map::<u32, _>(|(index, tiles)| {
                if tiles[tile_x].iter()
                                .any(|&tile| SOLID_TILES
                                .iter()
                                .any(|solid_ids| solid_ids.contains(&tile)))
                {
                    Some(index as u32)
                } else { None }
            }).unwrap_or(self.get_map_height() as u32);
        }

        if light_was_edited {
            for _ in 0..16 {
                for x in tile_x.saturating_sub(16 + edit_width)..(tile_x + 16 + edit_width).min(self.get_map_width() - 1) {
                    for y in min_edit_height.saturating_sub(16)..(max_edit_height + 16).min(self.get_map_height()) {
                        let left = self.get_light_mut(x.saturating_sub(1), y).iter().map(|f| f.saturating_sub(25)).collect::<Vec<u8>>();
                        let right = self.get_light_mut((x + 1).min(self.get_map_width() - 1), y).iter().map(|f| f.saturating_sub(25)).collect::<Vec<u8>>();
                        let up = self.get_light_mut(x, y.saturating_sub(1)).iter().map(|f| f.saturating_sub(25)).collect::<Vec<u8>>();
                        let down = self.get_light_mut(x, (y + 1).min(self.get_map_height() - 1)).iter().map(|f| f.saturating_sub(25)).collect::<Vec<u8>>();
                        let self_light = self.get_light_mut(x, y);
                        let light_level = left.iter()
                            .enumerate()
                            .map(|(i, f)| (*f).max(right[i].max(up[i].max(down[i].max(self_light[i])))))
                            .collect::<Vec<u8>>();
                        *self.get_light_mut(x, y) = [light_level[0], light_level[1], light_level[2]];
                    }
                }
            }
        }
        
        // updating the surrounding tiles (really ugly.... but works)
        for (x, y) in [(tile_x.saturating_sub(1), tile_y), (tile_x + 1, tile_y), (tile_x, tile_y.saturating_sub(1)), (tile_x, tile_y + 1)] {
            if x >= self.get_map_width() || tile_y >= self.get_map_height() { continue; }
            let tile = self.get_tile(x, y, layer);
            if GRASS_IDS.contains(&tile) {  // grass
                let tiles_outside = [
                    self.get_tile(x.saturating_sub(1), y, layer),
                    self.get_tile((x + 1).min(self.get_map_width() - 1), y, layer),
                    self.get_tile(x, y.saturating_sub(1), layer),
                    self.get_tile(x, (y + 1).min(self.get_map_height() - 1), layer),
                ];
                let tile_edges = [
                    tiles_outside[0] == 0,
                    tiles_outside[1] == 0,
                    tiles_outside[2] == 0,
                    tiles_outside[3] == 0,
                ];
                // left right up down
                let new_tile = match tile_edges {
                    [true, false, false, false] => 7,  // empty to left (wall facing to left)
                    [false, true, false, false] => 8,  // empty to right (wall facing to right)
                    [true, true, false, false]  => 10,  // empty to left and right (column)
                    [false, false, true, false] => 1,  // empty above (normal)
                    [false, false, false, true] => 47,  // empty below (upsidedown of normal)
                    [false, false, true, true]  => 5,  // empty above and below (ceiling ig)

                    [true, false, true, false]  => 2,  // empty to left and above (corner)
                    [true, false, false, true]  => 4,  // empty to left and below (corner)
                    [true, false, true, true]   => 13,  // empty to left above and below (cap facing left)

                    [false, true, false, true]  => 9,  // empty to right and below (corner)
                    [false, true, true, false]  => 3,  // empty to right and above (corner)
                    [false, true, true, true]   => 6,  // empty to right above and below (cap facing right)

                    [true, true, true, false]   => 11,  // empty to left, right and above (cap facing up)
                    [true, true, false, true]   => 12,  // empty to left, right and below (cap facing down)
                    [true, true, true, true]    => 14,  // surrounded

                    _ => 29,  // dirt
                };
                *self.get_tile_mut(x, y, layer) = new_tile;
            }

            if DIRT_IDS.contains(&tile) {  // dirt
                let tiles_outside = [
                    self.get_tile(x.saturating_sub(1), y, layer),
                    self.get_tile((x + 1).min(self.get_map_width() - 1), y, layer),
                    self.get_tile(x, y.saturating_sub(1), layer),
                    self.get_tile(x, (y + 1).min(self.get_map_height() - 1), layer),
                ];
                let tile_edges = [
                    tiles_outside[0] == 0,
                    tiles_outside[1] == 0,
                    tiles_outside[2] == 0,
                    tiles_outside[3] == 0,
                ];
                // left right up down
                let new_tile = match tile_edges {
                    [true, false, false, false] => 7  + 14,  // empty to left (wall facing to left)
                    [false, true, false, false] => 8  + 14,  // empty to right (wall facing to right)
                    [true, true, false, false]  => 10 + 14,  // empty to left and right (column)
                    [false, false, true, false] => 1  + 14,  // empty above (normal)
                    [false, false, false, true] => 46,       // empty below (upsidedown of normal)
                    [false, false, true, true]  => 5  + 14,  // empty above and below (ceiling ig)

                    [true, false, true, false]  => 2  + 14,  // empty to left and above (corner)
                    [true, false, false, true]  => 4  + 14,  // empty to left and below (corner)
                    [true, false, true, true]   => 13 + 14,  // empty to left above and below (cap facing left)

                    [false, true, false, true]  => 9  + 14,  // empty to right and below (corner)
                    [false, true, true, false]  => 3  + 14,  // empty to right and above (corner)
                    [false, true, true, true]   => 6  + 14,  // empty to right above and below (cap facing right)

                    [true, true, true, false]   => 11 + 14,  // empty to left, right and above (cap facing up)
                    [true, true, false, true]   => 12 + 14,  // empty to left, right and below (cap facing down)
                    [true, true, true, true]    => 14 + 14,  // surrounded

                    _ => 29,  // stone
                };
                *self.get_tile_mut(x, y, layer) = new_tile;
            }

            if STONE_IDS.contains(&tile) {  // stone
                let tiles_outside = [
                    self.get_tile(x.saturating_sub(1), y, layer),
                    self.get_tile((x + 1).min(self.get_map_width() - 1), y, layer),
                    self.get_tile(x, y.saturating_sub(1), layer),
                    self.get_tile(x, (y + 1).min(self.get_map_height() - 1), layer),
                ];
                let tile_edges = [
                    tiles_outside[0] == 0,
                    tiles_outside[1] == 0,
                    tiles_outside[2] == 0,
                    tiles_outside[3] == 0,
                ];
                // left right up down
                let new_tile = match tile_edges {
                    [true, false, false, false] => 7 + 29,  // empty to left (wall facing to left)
                    [false, true, false, false] => 8 + 29,  // empty to right (wall facing to right)
                    [true, true, false, false]  => 10 + 29,  // empty to left and right (column)
                    [false, false, true, false] => 1 + 29,  // empty above (normal)
                    [false, false, false, true] => 45,  // empty below (upsidedown of normal)
                    [false, false, true, true]  => 5 + 29,  // empty above and below (ceiling ig)

                    [true, false, true, false]  => 2 + 29,  // empty to left and above (corner)
                    [true, false, false, true]  => 4 + 29,  // empty to left and below (corner)
                    [true, false, true, true]   => 13 + 29,  // empty to left above and below (cap facing left)

                    [false, true, false, true]  => 9 + 29,  // empty to right and below (corner)
                    [false, true, true, false]  => 3 + 29,  // empty to right and above (corner)
                    [false, true, true, true]   => 6 + 29,  // empty to right above and below (cap facing right)

                    [true, true, true, false]   => 11 + 29,  // empty to left, right and above (cap facing up)
                    [true, true, false, true]   => 12 + 29,  // empty to left, right and below (cap facing down)
                    [true, true, true, true]    => 14 + 29,  // surrounded
                    
                    _ => 44,  // stone
                };
                *self.get_tile_mut(x, y, layer) = new_tile;
            }
        } Ok(())
    }
    
    // todo! fix the bug here that happens when zooming where the tiles jump around a bit, not sure where it is tbh
    pub fn get_render_slice(&mut self, camera_transform: &CameraTransform, window_size: (u32, u32)) -> (Vec<[u64; 4]>, CameraTransform, (u32, u32)) {
        // don't even try to read this or the math, it's a mess, but seems to work for now
        
        // the plus 2 is to make sure blocks at the very edge aren't cut off
        let visible_width = (window_size.0 as f32 / 8.0 * camera_transform.zoom) as usize + 4;
        let visible_height = (window_size.1 as f32 / 8.0 * camera_transform.zoom) as usize + 4;
        let start_x = ((camera_transform.x / 8.) as isize - (visible_width as isize / 2)).max(0) as usize;
        let start_y = ((camera_transform.y / 8.) as isize - (visible_height as isize / 2)).max(0) as usize;
        let end_x = (start_x + visible_width).min(self.get_map_width());
        let end_y = (start_y + visible_height).min(self.get_map_height());
        let cell_offset_x = (fract(camera_transform.x / 8.) * 8.) as f32 / camera_transform.zoom;
        let cell_offset_y = (fract(camera_transform.y / 8.) * 8.) as f32 / camera_transform.zoom;

        // generating the visible slice
        // the 1024 * 1024 is technically the max size as mandated by the gpu buffers
        
        // incase the camera goes out of bounds (to avoid weird visual bugs)
        if start_x >= end_x || start_y >= end_y {
            return (vec![], CameraTransform {
                x: 0.0,
                y: 0.0,
                zoom: camera_transform.zoom,
            }, (0, 0));
        }

        let mut visible_tiles: Vec<[u64; 4]> = Vec::with_capacity(((end_y - start_y) * (end_x - start_x)).max(0).min(1024 * 1024));
        for y in start_y..end_y {
            for x in start_x..end_x {
                let mut light = [self.lighting[y][x][0], self.lighting[y][x][1], self.lighting[y][x][2]];
                // going through entity lights and modifying it based on those
                for (_ident, light_obj) in &self.entity_lights {
                    let dif_x = light_obj.position.0 - (x as f32 * 8.0 + 4.0);
                    let dif_y = light_obj.position.1 - (y as f32 * 8.0 + 4.0);
                    if dif_x.abs() > 100.0 || dif_y.abs() > 100.0 { continue; }  // the light strength would be 0
                    let distance = (dif_x * dif_x + dif_y * dif_y) * 0.00025;
                    let light_strength = (light_obj.color.3 - distance).max(0.0);
                    light = [
                        light[0].max((light_obj.color.0 as f32 * light_strength) as u8),
                        light[1].max((light_obj.color.1 as f32 * light_strength) as u8),
                        light[2].max((light_obj.color.2 as f32 * light_strength) as u8),
                    ];
                }
                
                // computing the light contribution from the sky
                let mut sky_light = 0;
                for x_offset in (-10isize)..10isize {
                    let sky_light_new = 10usize.saturating_sub(y.saturating_sub(self.sky_light[(x as isize + x_offset).max(0) as usize] as usize));
                    // the pow is to create an easing curve to make it less diamond shaped, but idk how I feel about it. But, for now, it works
                    let sky_light_new = ((sky_light_new as f32 / 10.0 * 255.0) as u8).saturating_sub(((x_offset as f32 * 0.1).powi(2) * 255.0) as u8);
                    sky_light = sky_light.max(sky_light_new);
                }

                // getting the final lighting for the location
                light = [
                    light[0].max(sky_light),
                    light[1].max(sky_light),
                    light[2].max(sky_light),
                ];
                self.mini_map.update_light_value(light, x, y);

                visible_tiles.push([
                    self.tiles[y][x][0] as u64,
                    self.tiles[y][x][1] as u64,
                    self.tiles[y][x][2] as u64,
                    (
                        light[0] as u32 |
                        ((light[1] as u32) << 8) |
                        ((light[2] as u32) << 16)
                    ) as u64,
                ]);
            }
        }

        let edge_offset_x = ((camera_transform.x / 8.) as isize - (visible_width as isize / 2)).min(0) as f32 * -8.0 / camera_transform.zoom;
        let edge_offset_y = ((camera_transform.y / 8.) as isize - (visible_height as isize / 2)).min(0) as f32 * -8.0 / camera_transform.zoom;
        (visible_tiles, CameraTransform {
            x: edge_offset_x - cell_offset_x,
            y: edge_offset_y - cell_offset_y,
            zoom: camera_transform.zoom,
        }, ((end_x - start_x) as u32, (end_y - start_y) as u32))
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct TileMapManager {
    tile_maps: [Option<TileMap>; Dimension::TOTAL as usize],
}

impl TileMapManager {
    pub fn new() -> Self {
        TileMapManager {
            tile_maps: [None; Dimension::TOTAL as usize],
        }
    }

    pub fn get_current_map(&mut self, dimension: Dimension) -> Option<&mut TileMap> {
        let index = dimension as usize;
        self.tile_maps[index].as_mut()
    }

    pub fn get_current_map_ref(&self, dimension: Dimension) -> Option<&TileMap> {
        let index = dimension as usize;
        self.tile_maps[index].as_ref()
    }

    pub fn replace_tile_map(&mut self, dimension: Dimension, tile_map: TileMap) {
        let index = dimension as usize;
        self.tile_maps[index] = Some(tile_map);
    }
}

#[repr(u32)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum Dimension {
    Overworld = 0,
    
    TOTAL = 1,
}
