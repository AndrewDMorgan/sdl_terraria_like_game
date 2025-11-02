
pub struct WorldGenerator {
    //
}

impl WorldGenerator {
    pub fn new() -> Self {
        WorldGenerator {
            //
        }
    }

    // todo! add perlin noise and stuff
    pub fn generate_tile_map(&self, tile_map: &mut crate::game_manager::world::tile_map::TileMap) {
        for x in 0..tile_map.get_map_width() {
            for y in 0..tile_map.get_map_height() {
                // simple flat world for now
                if y < 128 {
                    if y != 127 { continue; }
                    *tile_map.get_tile_mut(x, y, 0) = 1; // grass
                } else if y < 136 {
                    *tile_map.get_tile_mut(x, y, 0)  = 29; // dirt
                } else {
                    *tile_map.get_tile_mut(x, y, 0) = 44; // stone
                }
                // todo! temporary, this is just so things are actually visible
                *tile_map.get_light_mut(x, y) = [255, 255, 255];
            }
        }
    }
}

