use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

use crate::game_manager::world::tile_map::{DIRT_IDS, GRASS_IDS, STONE_IDS};


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
        cave_noise.set_noise_type(Some(NoiseType::ValueCubic));
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

            let mut light_level: u8 = 250;
            for y in 0..tile_map.get_map_height() {
                let height = ((noise.get_noise_2d(x as f32, y as f32) * 0.5 + 0.5) * 50.0 + 100.0) as usize;
                let cave_noise = cave_noise.get_noise_2d(x as f32, y as f32);

                if cave_noise > cave_threshold_noise.get_noise_2d(x as f32, y as f32) + 1.5 + ((y as f32 - height as f32) * -0.1).max(-0.75) {
                    *tile_map.get_light_mut(x, y) = [light_level, light_level, light_level];
                    continue;
                }
                
                // simple flat world for now
                if y == height {
                    *tile_map.get_tile_mut(x, y, 0) = 1;
                    light_level = light_level.saturating_sub(25);
                } else if y > height && y <= height + dirt_depth {
                    *tile_map.get_tile_mut(x, y, 0) = 29;  // dirt
                    light_level = light_level.saturating_sub(25);
                } else if y > height + dirt_depth {
                    *tile_map.get_tile_mut(x, y, 0) = 44; // stone
                    light_level = light_level.saturating_sub(25);
                }
                *tile_map.get_light_mut(x, y) = [light_level, light_level, light_level];
            }
        }
        
        for _ in 0..16 {
            for x in 0..tile_map.get_map_width() {
                for y in 0..tile_map.get_map_height() {
                    let left = tile_map.get_light_mut(x.saturating_sub(1), y)[0].saturating_sub(25);
                    let right = tile_map.get_light_mut((x + 1).min(tile_map.get_map_width() - 1), y)[0].saturating_sub(25);
                    let up = tile_map.get_light_mut(x, y.saturating_sub(1))[0].saturating_sub(25);
                    let down = tile_map.get_light_mut(x, (y + 1).min(tile_map.get_map_height() - 1))[0].saturating_sub(25);
                    let self_light = tile_map.get_light_mut(x, y)[0];
                    let light_level = left.max(right.max(up.max(down.max(self_light))));
                    *tile_map.get_light_mut(x, y) = [light_level, light_level, light_level];
                }
            }
        }

        // post processing the dirt and grass to make them prettier
        // this should work better as it should support things like cave cutouts and stuff
        for x in 0..tile_map.get_map_width() {
            for y in 0..tile_map.get_map_height() {
                let tile = tile_map.get_tile(x, y, 0);
                if GRASS_IDS.contains(&tile) || DIRT_IDS.contains(&tile) {  // grass
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


                if STONE_IDS.contains(&tile) {  // stone
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

        // trying to generate trees and bushes in flat sections
        // todo! make a much better system for predefined structures or multi-block objects
        for x in 4..tile_map.get_map_width() - 4 {
            let tree_chance = x % 25;
            if tree_chance != 10 && tree_chance != 20 { continue; }
            // locating the top top surface
            for y in 4..tile_map.get_map_height() - 4 {
                let tile = tile_map.get_tile(x, y, 0);
                if tile == 1 {
                    if tree_chance == 10 {
                        // bush
                        // clear to left and right is required
                        let left_tile = tile_map.get_tile(x.saturating_sub(1), y, 0);
                        let right_tile = tile_map.get_tile((x + 1).min(tile_map.get_map_width() - 1), y, 0);
                        if left_tile != 1 || right_tile != 1 { break; }
                        // + 48
                        // _  25 26 27
                        // 30 31 32 33
                        // 36 37 38 39
                        *tile_map.get_tile_mut(x + 0, y - 2, 1) = 25 + 48;
                        *tile_map.get_tile_mut(x + 1, y - 2, 1) = 26 + 48;
                        *tile_map.get_tile_mut(x + 2, y - 2, 1) = 27 + 48;

                        *tile_map.get_tile_mut(x - 1, y - 1, 1) = 30 + 48;
                        *tile_map.get_tile_mut(x + 0, y - 1, 1) = 31 + 48;
                        *tile_map.get_tile_mut(x + 1, y - 1, 1) = 32 + 48;
                        *tile_map.get_tile_mut(x + 2, y - 1, 1) = 33 + 48;

                        *tile_map.get_tile_mut(x - 1, y, 1) = 36 + 48;
                        *tile_map.get_tile_mut(x + 0, y, 1) = 37 + 48;
                        *tile_map.get_tile_mut(x + 1, y, 1) = 38 + 48;
                        *tile_map.get_tile_mut(x + 2, y, 1) = 39 + 48;
                    } else {
                        // tree + 48
                        // _  0  1  2  3
                        // 4  5  6  7  8
                        // 9  10 11 12 13
                        // _  14 15 16 17
                        // _  18 19 20 _
                        // _  21 22 23 _
                        // _  _  24 _  _
                        // _  _  28 29 _
                        // _  _  34 35 _
                        *tile_map.get_tile_mut(x - 1, y - 8, 1) = 48 + 0;
                        *tile_map.get_tile_mut(x + 0, y - 8, 1) = 48 + 1;
                        *tile_map.get_tile_mut(x + 1, y - 8, 1) = 48 + 2;
                        *tile_map.get_tile_mut(x + 2, y - 8, 1) = 48 + 3;

                        *tile_map.get_tile_mut(x - 2, y - 7, 1) = 48 + 4;
                        *tile_map.get_tile_mut(x - 1, y - 7, 1) = 48 + 5;
                        *tile_map.get_tile_mut(x + 0, y - 7, 1) = 48 + 6;
                        *tile_map.get_tile_mut(x + 1, y - 7, 1) = 48 + 7;
                        *tile_map.get_tile_mut(x + 2, y - 7, 1) = 48 + 8;

                        *tile_map.get_tile_mut(x - 2, y - 6, 1) = 48 + 9;
                        *tile_map.get_tile_mut(x - 1, y - 6, 1) = 48 + 10;
                        *tile_map.get_tile_mut(x + 0, y - 6, 1) = 48 + 11;
                        *tile_map.get_tile_mut(x + 1, y - 6, 1) = 48 + 12;
                        *tile_map.get_tile_mut(x + 2, y - 6, 1) = 48 + 13;

                        *tile_map.get_tile_mut(x - 1, y - 5, 1) = 48 + 14;
                        *tile_map.get_tile_mut(x + 0, y - 5, 1) = 48 + 15;
                        *tile_map.get_tile_mut(x + 1, y - 5, 1) = 48 + 16;
                        *tile_map.get_tile_mut(x + 2, y - 5, 1) = 48 + 17;

                        *tile_map.get_tile_mut(x - 1, y - 4, 1) = 48 + 18;
                        *tile_map.get_tile_mut(x + 0, y - 4, 1) = 48 + 19;
                        *tile_map.get_tile_mut(x + 1, y - 4, 1) = 48 + 20;

                        *tile_map.get_tile_mut(x - 1, y - 3, 1) = 48 + 21;
                        *tile_map.get_tile_mut(x + 0, y - 3, 1) = 48 + 22;
                        *tile_map.get_tile_mut(x + 1, y - 3, 1) = 48 + 23;

                        *tile_map.get_tile_mut(x + 0, y - 2, 1) = 48 + 24;

                        *tile_map.get_tile_mut(x + 0, y - 1, 1) = 48 + 28;
                        *tile_map.get_tile_mut(x + 1, y - 1, 1) = 48 + 29;

                        *tile_map.get_tile_mut(x + 0, y - 0, 1) = 48 + 34;
                        *tile_map.get_tile_mut(x + 1, y - 0, 1) = 48 + 35;
                    }
                }
                if tile != 0 { break; }
            }
        }

    }
}

