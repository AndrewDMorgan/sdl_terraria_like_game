use crate::core::event_handling::event_handler::{ButtonState, EventHandler};
use crate::game_manager::entities::entity::Entity;
use crate::game_manager::entities::manager::EntityManager;
use crate::game_manager::entities::player::inventory::Inventory;
use crate::game_manager::entities::player::player_ui::PlayerUiManager;
use crate::game_manager::game::GameError;
use crate::game_manager::world::tile_map;
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KeyBindings {
    pub inventory: Vec<KeyBind>,
    pub left: Vec<KeyBind>,
    pub right: Vec<KeyBind>,
    pub jump: Vec<KeyBind>,
    pub down: Vec<KeyBind>,
}

impl KeyBindings {
    pub fn check_true(binding: &Vec<KeyBind>, raw_keys: &Vec<i32>, key_mods: &Vec<sdl2::keyboard::Mod>) -> bool {
        binding.iter().any(|k| match k {
            KeyBind::Key(key) => raw_keys.contains(key),
            KeyBind::Mod(mods) => key_mods.iter().any(|m| m.bits() == *mods),
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum KeyBind {
    Key(i32),
    Mod(u16),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlayerData {
    pub inventory: Inventory,
}

/// The player entity module
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub camera: CameraTransform,
    pub entity: Entity<PlayerAnimation>,
    player_data: PlayerData,
    key_bindings: KeyBindings,
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
                    offset: (-12.0, -9.0),
                    size: (8.0, 14.0),
                }, Animator::new(vec![vec![0]], vec![0.0])),
                position: (0.0, 115.0 * 8.0),  // this should be on the edge of the map, but it's not, so that's an issue that needs addressing and probably relates to the zooming bug
            },
            player_data: PlayerData {
                inventory: Inventory::new(),
            },
            key_bindings: KeyBindings {
                inventory: vec![KeyBind::Key(*sdl2::keyboard::Keycode::E)],
                left: vec![KeyBind::Key(*sdl2::keyboard::Keycode::A)],
                right: vec![KeyBind::Key(*sdl2::keyboard::Keycode::D)],
                jump: vec![KeyBind::Key(*sdl2::keyboard::Keycode::W)],
                down: vec![KeyBind::Mod(sdl2::keyboard::Mod::LSHIFTMOD.bits()), KeyBind::Key(*sdl2::keyboard::Keycode::S)],
            },
        }
    }

    pub fn render_ui(&mut self, buffer: &mut [u8], buffer_size: (u32, u32), player_ui_manager: &mut PlayerUiManager, pitch: usize) -> Result<(), crate::core::rendering::ui::UiError> {
        player_ui_manager.render_ui(buffer, buffer_size, &mut self.player_data, pitch)
    }

    fn move_player(&mut self, delta_x: f32, delta_y: f32, tile_map: &tile_map::TileMap) {
        // trying to do a smoother collision detection by splitting the movement into many steps
        let hitbox = self.entity.sprite.get_hitbox();
        for _ in 0..100 {
            let new_x = self.entity.position.0 + delta_x * 0.01;
            let new_y = self.entity.position.1 + delta_y * 0.01;

            if tile_map.check_aabb_collision(new_x + hitbox.offset.0 as f32, new_y + hitbox.offset.1 as f32, hitbox.size.0 as f32, hitbox.size.1 as f32) {
                if delta_x != 0.0 && delta_y.abs() <= 0.01 {
                    let mut jumped = false;
                    // trying to jump up the block if it's 1 high
                    for y in 1..=16 {
                        let test_y = new_y - y as f32 * 0.5;
                        if !tile_map.check_aabb_collision(new_x + hitbox.offset.0 as f32, test_y + hitbox.offset.1 as f32, hitbox.size.0 as f32, hitbox.size.1 as f32) {
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

    pub fn update_key_events(
        &mut self,
        timer: &Timer,
        event_handler: &EventHandler,
        tile_map: &mut tile_map::TileMap,
        screen_size: (u32, u32),
        ui_manager: &mut PlayerUiManager,
        entity_manager: &mut EntityManager,
        rand_state: &mut dyn rand::RngCore,
    ) -> Result<(), GameError> {
        self.entity.sprite.update_frame(timer.delta_time);  // this is the best place to do this ig

        self.player_data.inventory.update_key_events(
            timer,
            event_handler,
            tile_map,
            screen_size,
            &self.key_bindings,
            ui_manager,
            entity_manager,
            &self.entity.position,
        )?;

        let raw_keys_held = event_handler.keys_held.iter().map(|k| **k).collect::<Vec<_>>();
        if KeyBindings::check_true(&self.key_bindings.right, &raw_keys_held, &event_handler.mods_held) {
            self.move_player(200.0 * timer.delta_time as f32, 0.0, tile_map);
        }
        if KeyBindings::check_true(&self.key_bindings.left, &raw_keys_held, &event_handler.mods_held) {
            self.move_player(-200.0 * timer.delta_time as f32, 0.0, tile_map);
        }
        if KeyBindings::check_true(&self.key_bindings.down, &raw_keys_held, &event_handler.mods_held) {
            self.move_player(0.0, 200.0 * timer.delta_time as f32, tile_map);
        }
        if KeyBindings::check_true(&self.key_bindings.jump, &raw_keys_held, &event_handler.mods_held) {
            self.move_player(0.0, -200.0 * timer.delta_time as f32, tile_map);
        }

        if event_handler.keys_held.contains(&sdl2::keyboard::Keycode::Z) {
            if event_handler.mods_held.contains(&sdl2::keyboard::Mod::LALTMOD) {
                self.camera.zoom += 0.075 * timer.delta_time as f32;
            } else {
                self.camera.zoom -= 0.075 * timer.delta_time as f32;
            }
        }

        // tempory tile deletion
        if let ButtonState::Pressed | ButtonState::Held = event_handler.mouse.left {
            let mouse_x = self.camera.x - screen_size.0 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.0 as f32 * self.camera.zoom;
            let mouse_y = self.camera.y - screen_size.1 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.1 as f32 * self.camera.zoom;
            let tile_x = (mouse_x / 8.0 - 1.0).floor() as usize;
            let tile_y = (mouse_y / 8.0 - 0.5).floor() as usize;
            if tile_x < tile_map.get_map_width() && tile_y < tile_map.get_map_height() {
                self.player_data.inventory.left_click_item(tile_x, tile_y, tile_map, event_handler, ui_manager, entity_manager, rand_state)?;
            }
        }
        if let ButtonState::Pressed | ButtonState::Held = event_handler.mouse.right {
            let mouse_x = self.camera.x - screen_size.0 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.0 as f32 * self.camera.zoom;
            let mouse_y = self.camera.y - screen_size.1 as f32 * 0.5 * self.camera.zoom + event_handler.mouse.position.1 as f32 * self.camera.zoom;
            let tile_x = (mouse_x / 8.0 - 1.0).floor() as usize;
            let tile_y = (mouse_y / 8.0 - 0.5).floor() as usize;
            if tile_x < tile_map.get_map_width() && tile_y < tile_map.get_map_height() {
                self.player_data.inventory.right_click_item(tile_x, tile_y, tile_map, event_handler, ui_manager, entity_manager)?;
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

        if let Some(entity_light) = tile_map.entity_lights.iter_mut().find(|(ident,_)| &**ident == "Player") {
            entity_light.1.position = (self.entity.position.0 - 4.0, self.entity.position.1 - 4.0);
        }

        Ok(())
    }
    
    // the entities are a 128 bit value, with the first 32 being the texture id (similar to tiles),
    // the next 16 being rotation ( f16 of [0, pi) ), and 16 for x + 16 for y (screen space offsets, uint), with 44 bits for applicable data, and the
    // the 4 being depth (to correctly layer them; hopefully 4 bits is enough, but idk)
    pub fn get_model(&self) -> Vec<(u32, u16, i16, i16, u16, u32)> {
        // the player is divided into 6 parts as entity textures can only be 8x8
        vec![  // todo! move the sprite indexing into the sprite struct and animator
            (
                self.entity.sprite.get_texture() + 1, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 800,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 - 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 2, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 - 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 3, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 800,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 4, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 5, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16 - 800,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 + 800,
                0, 15,
            ),
            (
                self.entity.sprite.get_texture() + 6, 0,
                ((self.entity.position.0 - self.camera.x) * 100.0) as i16,
                ((self.entity.position.1 - self.camera.y) * 100.0) as i16 + 800,
                0, 15,
            ),
        ]
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CameraTransform {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

