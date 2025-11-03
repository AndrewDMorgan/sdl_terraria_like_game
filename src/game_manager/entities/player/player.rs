use crate::core::event_handling::event_handler::EventHandler;
use crate::core::timer::Timer;
use crate::game_manager::world::tile_map;

/// The player entity module
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub camera: CameraTransform,
    player_position: (f32, f32),

    // todo! move the player model into an 'Entity' struct later that'll manage entities
    // this should remove some of the dependencies that have been injected into here
}

impl Player {
    pub fn new() -> Self {
        Player {
            camera: CameraTransform {
                x: 0.0,
                y: 135.0 * 8.0,
                zoom: 0.2,
            },
            player_position: (-20.0, 135.0 * 8.0),
        }
    }

    fn move_player(&mut self, delta_x: f32, delta_y: f32, tile_map: &tile_map::TileMap) {
        // trying to do a smoother collision detection by splitting the movement into 10 steps
        for _ in 0..10 {
            let new_x = self.player_position.0 + delta_x * 0.1;
            let new_y = self.player_position.1 + delta_y * 0.1;

            if tile_map.check_aabb_collision(new_x - 8.0, new_y - 9.0, 8.0, 14.0) {
                break;
            } else {
                self.player_position.0 = new_x;
                self.player_position.1 = new_y;
            }
        }
    }

    pub fn update_key_events(&mut self, timer: &Timer, event_handler: &EventHandler, tile_map: &tile_map::TileMap) {
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Right) {
            self.move_player(200.0 * timer.delta_time as f32, 0.0, tile_map);
            //self.player_position.0 += 200.0 * timer.delta_time as f32;
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Left) {
            self.move_player(-200.0 * timer.delta_time as f32, 0.0, tile_map);
            //self.player_position.0 -= 200.0 * timer.delta_time as f32;
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Down) {
            self.move_player(0.0, 200.0 * timer.delta_time as f32, tile_map);
            //self.player_position.1 += 200.0 * timer.delta_time as f32;
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Up) {
            self.move_player(0.0, -200.0 * timer.delta_time as f32, tile_map);
            //self.player_position.1 -= 200.0 * timer.delta_time as f32;
        }

        // smooth camera movement!
        self.camera.x = lerp(
            self.camera.x,
            self.player_position.0,
            10.0 * timer.delta_time as f32,
        );
        self.camera.y = lerp(
            self.camera.y,
            self.player_position.1,
            10.0 * timer.delta_time as f32,
        );
    }
    
    // the entities are a 128 bit value, with the first 32 being the texture id (similar to tiles),
    // the next 16 being rotation ( f16 of [0, pi) ), and 16 for x + 16 for y (screen space offsets, uint), with 44 bits for applicable data, and the
    // the 4 being depth (to correctly layer them; hopefully 4 bits is enough, but idk)
    pub fn get_model(&self) -> Vec<(u32, u16, i16, i16, u16, u32)> {
        // the player is divided into 6 parts as entity textures can only be 8x8
        vec![
            (
                1, 0,
                ((self.player_position.0 - self.camera.x)) as i16 - 4,
                ((self.player_position.1 - self.camera.y)) as i16 - 8,
                0, 15,
            ),
            (
                2, 0,
                ((self.player_position.0 - self.camera.x)) as i16 + 4,
                ((self.player_position.1 - self.camera.y)) as i16 - 8,
                0, 15,
            ),
            (
                3, 0,
                ((self.player_position.0 - self.camera.x)) as i16 - 4,
                ((self.player_position.1 - self.camera.y)) as i16,
                0, 15,
            ),
            (
                4, 0,
                ((self.player_position.0 - self.camera.x)) as i16 + 4,
                ((self.player_position.1 - self.camera.y)) as i16,
                0, 15,
            ),
            (
                5, 0,
                ((self.player_position.0 - self.camera.x)) as i16 - 4,
                ((self.player_position.1 - self.camera.y)) as i16 + 8,
                0, 15,
            ),
            (
                6, 0,
                ((self.player_position.0 - self.camera.x)) as i16 + 4,
                ((self.player_position.1 - self.camera.y)) as i16 + 8,
                0, 15,
            ),
        ]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CameraTransform {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

