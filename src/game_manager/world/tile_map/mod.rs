
use crate::game_manager::entities::player::player::CameraTransform;
use crate::game_manager::world::world_gen::WorldGenerator;

pub static GRASS_IDS: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
pub static DIRT_IDS: &[u32] = &[15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29];
pub static STONE_IDS: &[u32] = &[30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44];

pub static SOLID_TILES: &[&[u32]] = &[
    GRASS_IDS,
    DIRT_IDS,
    STONE_IDS,
];

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

    pub fn get_tile(&self, x: usize, y: usize, layer: usize) -> u32 {
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

    pub fn get_render_slice(&self, camera_transform: &CameraTransform, window_size: (u32, u32)) -> (Vec<[u64; 4]>, CameraTransform, (u32, u32)) {
        // don't even try to read this or the math, it's a mess, but seems to work for now
        
        // the plus 2 is to make sure blocks at the very edge aren't cut off
        let visible_width = (window_size.0 as f32 / 8.0 * camera_transform.zoom) as usize + 2;
        let visible_height = (window_size.1 as f32 / 8.0 * camera_transform.zoom) as usize + 2;
        let start_x = ((camera_transform.x / 8.) as isize - (visible_width as isize / 2)).max(0) as usize;
        let start_y = ((camera_transform.y / 8.) as isize - (visible_height as isize / 2)).max(0) as usize;
        let end_x = (start_x + visible_width).min(self.get_map_width());
        let end_y = (start_y + visible_height).min(self.get_map_height());
        let cell_offset_x = (fract(camera_transform.x / 8.) * 8.) as f32 / camera_transform.zoom;
        let cell_offset_y = (fract(camera_transform.y / 8.) * 8.) as f32 / camera_transform.zoom;

        // generating the visible slice
        let mut visible_tiles: Vec<[u64; 4]> = Vec::with_capacity((end_y - start_y) * (end_x - start_x));
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
pub enum Dimension {
    Overworld = 0,
    
    TOTAL = 1,
}
