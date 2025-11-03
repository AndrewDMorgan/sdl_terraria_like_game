use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};


#[derive(serde::Serialize, serde::Deserialize)]
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
        let mut noise = FastNoiseLite::new();
        noise.set_seed(Some(1337));
        noise.set_noise_type(Some(NoiseType::Perlin));
        noise.set_frequency(Some(0.02));
        noise.set_fractal_type(Some(FractalType::FBm));
        noise.set_fractal_octaves(Some(5));
        noise.set_fractal_lacunarity(Some(2.0));
        noise.set_fractal_gain(Some(0.5));


        let mut cave_noise = FastNoiseLite::new();
        cave_noise.set_seed(Some(42069));
        cave_noise.set_noise_type(Some(NoiseType::Cellular));
        cave_noise.set_frequency(Some(0.05));
        cave_noise.set_cellular_distance_function(Some(fastnoise_lite::CellularDistanceFunction::Manhattan));
        cave_noise.set_cellular_return_type(Some(fastnoise_lite::CellularReturnType::Distance2Div));
        cave_noise.set_fractal_type(Some(FractalType::Ridged));
        cave_noise.set_fractal_octaves(Some(3));
        cave_noise.set_fractal_lacunarity(Some(2.0));
        cave_noise.set_fractal_gain(Some(0.5));


        let mut cave_threshold_noise = FastNoiseLite::new();
        cave_threshold_noise.set_seed(Some(9876));
        cave_threshold_noise.set_noise_type(Some(NoiseType::Perlin));
        cave_threshold_noise.set_frequency(Some(0.1));

        
        for x in 0..tile_map.get_map_width() {
            let dirt_depth = ((noise.get_noise_2d(x as f32, 25.0) * 0.5 + 0.5) * 10.0) as usize;


            for y in 0..tile_map.get_map_height() {
                let height = ((noise.get_noise_2d(x as f32, y as f32) * 0.5 + 0.5) * 50.0 + 100.0) as usize;
                let cave_noise = cave_noise.get_noise_2d(x as f32, y as f32);

                if cave_noise > cave_threshold_noise.get_noise_2d(x as f32, y as f32) + 0.5 {
                    continue;
                }
                
                // simple flat world for now
                if y == height {
                    *tile_map.get_tile_mut(x, y, 0) = 1;
                } else if y > height && y <= height + dirt_depth {
                    *tile_map.get_tile_mut(x, y, 0) = 29  // dirt
                } else if y > height + dirt_depth {
                    *tile_map.get_tile_mut(x, y, 0) = 44; // stone
                }
                // todo! temporary, this is just so things are actually visible
                *tile_map.get_light_mut(x, y) = [255, 255, 255];
            }
        }

        // post processing the dirt and grass to make them prettier
        // this should work better as it should support things like cave cutouts and stuff
        for x in 0..tile_map.get_map_width() {
            for y in 0..tile_map.get_map_height() {
                let tile = tile_map.get_tile(x, y, 0);
                if [1, 29].contains(&tile) {  // grass
                    let tiles_outside = [
                        tile_map.get_tile(x.saturating_sub(1), y, 0),
                        tile_map.get_tile((x + 1).min(tile_map.get_map_width() - 1), y, 0),
                        tile_map.get_tile(x, y.saturating_sub(1), 0),
                        tile_map.get_tile(x, (y + 1).min(tile_map.get_map_height() - 1), 0),
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
                    *tile_map.get_tile_mut(x, y, 0) = new_tile;
                }


                if tile == 44 {  // stone
                    let tiles_outside = [
                        tile_map.get_tile(x.saturating_sub(1), y, 0),
                        tile_map.get_tile((x + 1).min(tile_map.get_map_width() - 1), y, 0),
                        tile_map.get_tile(x, y.saturating_sub(1), 0),
                        tile_map.get_tile(x, (y + 1).min(tile_map.get_map_height() - 1), 0),
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
                    *tile_map.get_tile_mut(x, y, 0) = new_tile;
                }
            }
        }

    }
}

