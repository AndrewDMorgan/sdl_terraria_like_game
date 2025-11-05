
use crate::game_manager::entities::player::player::CameraTransform;
use crate::game_manager::world::world_gen::WorldGenerator;

pub static GRASS_IDS: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 47];
pub static DIRT_IDS: &[u32] = &[15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 46];
pub static STONE_IDS: &[u32] = &[30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45];

pub static SOLID_TILES: &[&[u32]] = &[
    GRASS_IDS,
    DIRT_IDS,
    STONE_IDS,
];

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TileMap {
    tiles: Vec<Vec<[u32; 3]>>,
    lighting: Vec<Vec<[u8; 3]>>,
}

impl TileMap {
    pub fn new(width: usize, height: usize, world_generator: Option<&WorldGenerator>) -> Self {
        let mut tile_map = TileMap {
            tiles: vec![vec![[0; 3]; width]; height],
            lighting: vec![vec![[0; 3]; width]; height],
        };
        if let Some(generator) = world_generator {
            generator.generate_tile_map(&mut tile_map);
        }
        tile_map
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

    pub fn change_tile(&mut self, tile_x: usize, tile_y: usize, layer: usize, new_tile: u32) {
        *self.get_tile_mut(tile_x, tile_y, layer) = new_tile;
        
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
        }
    }

    pub fn get_render_slice(&self, camera_transform: &CameraTransform, window_size: (u32, u32)) -> (Vec<[u64; 4]>, CameraTransform, (u32, u32)) {
        // don't even try to read this or the math, it's a mess, but seems to work for now
        
        // the plus 2 is to make sure blocks at the very edge aren't cut off
        let visible_width = (window_size.0 as f32 / 8.0 * camera_transform.zoom) as usize + 3;
        let visible_height = (window_size.1 as f32 / 8.0 * camera_transform.zoom) as usize + 3;
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
                visible_tiles.push([
                    self.tiles[y][x][0] as u64,
                    self.tiles[y][x][1] as u64,
                    self.tiles[y][x][2] as u64,
                    (
                        self.lighting[y][x][0] as u32 |
                        ((self.lighting[y][x][1] as u32) << 8) |
                        ((self.lighting[y][x][2] as u32) << 16)
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

#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Dimension {
    Overworld = 0,
    
    TOTAL = 1,
}
