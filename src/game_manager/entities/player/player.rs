use crate::core::event_handling::event_handler::{ButtonState, EventHandler};
use crate::game_manager::entities::entity::Entity;
use crate::game_manager::world::tile_map::{self, DIRT_IDS, GRASS_IDS, STONE_IDS};
use crate::core::timer::Timer;
use crate::textures::animation::Animator;
use crate::textures::sprite::{Hitbox, Sprite};

#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Default)]
pub enum PlayerAnimation {
    #[default] Idle = 0,
}

impl Into<u8> for PlayerAnimation {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for PlayerAnimation {
    fn from(value: u8) -> Self {
        match value {
            0 => PlayerAnimation::Idle,
            _ => PlayerAnimation::Idle,
        }
    }
}


/// The player entity module
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub camera: CameraTransform,
    pub entity: Entity<PlayerAnimation>,

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
            entity: Entity {
                sprite: Sprite::new_animated((8.0, 14.0), (-8.0, -9.0), vec![2, 2, 2], Hitbox {
                    offset: (-8.0, -9.0),
                    size: (8.0, 14.0),
                }, Animator::new(vec![vec![0]], vec![0.0])),
                position: (-20.0, 115.0 * 8.0),
            },
        }
    }

    fn move_player(&mut self, delta_x: f32, delta_y: f32, tile_map: &tile_map::TileMap) {
        // trying to do a smoother collision detection by splitting the movement into many steps
        for _ in 0..100 {
            let new_x = self.entity.position.0 + delta_x * 0.01;
            let new_y = self.entity.position.1 + delta_y * 0.01;

            if tile_map.check_aabb_collision(new_x - 8.0, new_y - 9.0, 8.0, 14.0) {
                if delta_x != 0.0 && delta_y.abs() <= 0.01 {
                    let mut jumped = false;
                    // trying to jump up the block if it's 1 high
                    for y in 1..=16 {
                        let test_y = new_y - y as f32 * 0.5;
                        if !tile_map.check_aabb_collision(new_x - 8.0, test_y - 9.0, 8.0, 14.0) {
                            self.entity.position.0 = new_x;
                            self.entity.position.1 = test_y;
                            jumped = true;
                            break;
                        }
                    }
                    if !jumped { break; }
                } else { break; }
            } else {
                self.entity.position.0 = new_x;
                self.entity.position.1 = new_y;
            }
        }
    }

    pub fn update_key_events(&mut self, timer: &Timer, event_handler: &EventHandler, tile_map: &mut tile_map::TileMap, screen_size: (u32, u32)) {
        self.entity.sprite.update_frame(timer.delta_time);  // this is the best place to do this ig

        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Right) {
            self.move_player(200.0 * timer.delta_time as f32, 0.0, tile_map);
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Left) {
            self.move_player(-200.0 * timer.delta_time as f32, 0.0, tile_map);
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Down) {
            self.move_player(0.0, 200.0 * timer.delta_time as f32, tile_map);
        }
        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Up) {
            self.move_player(0.0, -200.0 * timer.delta_time as f32, tile_map);
        }

        // tempory tile deletion
        if let ButtonState::Pressed | ButtonState::Held = event_handler.mouse.left {
            let mouse_x = self.camera.x - screen_size.0 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.0 as f32 * self.camera.zoom;
            let mouse_y = self.camera.y - screen_size.1 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.1 as f32 * self.camera.zoom;
            let tile_x = (mouse_x / 8.0 - 1.0).floor() as usize;
            let tile_y = (mouse_y / 8.0 - 0.5).floor() as usize;
            if tile_x < tile_map.get_map_width() && tile_y < tile_map.get_map_height() {
                tile_map.change_tile(tile_x, tile_y, 0, 0);
            }
        }

        // smooth camera movement!
        self.camera.x = lerp(
            self.camera.x,
            self.entity.position.0,
            10.0 * timer.delta_time as f32,
        );
        self.camera.y = lerp(
            self.camera.y,
            self.entity.position.1,
            10.0 * timer.delta_time as f32,
        );
    }
    
    // the entities are a 128 bit value, with the first 32 being the texture id (similar to tiles),
    // the next 16 being rotation ( f16 of [0, pi) ), and 16 for x + 16 for y (screen space offsets, uint), with 44 bits for applicable data, and the
    // the 4 being depth (to correctly layer them; hopefully 4 bits is enough, but idk)
    pub fn get_model(&self) -> Vec<(u32, u16, i16, i16, u16, u32)> {
        // the player is divided into 6 parts as entity textures can only be 8x8
        vec![  // todo! move the sprite indexing into the sprite struct and animator
            (
                self.entity.sprite.get_texture() + 1, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 - 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 2, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 + 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 - 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 3, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 4, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 + 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 5, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 + 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 6, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 + 400,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 + 800,
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

