use std::rc::Rc;

use crate::game_manager::entities::manager::EntityManager;
use crate::game_manager::entities::player::{player::*, player_ui::PlayerUiManager};
use crate::game_manager::world::{world_gen::*, tile_map::*};
use crate::shaders::shader_loader::MAX_ENTITIES;
use crate::textures::textures::get_texture_atlas;
use crate::logging::logging::{Log, LogType, Logs};

// the maximum item textures (in this context, this can safely be set to any value >= to the total texture count as no gpu buffers rely upon a static size for this)
static MAX_ITEM_TEXTURES: usize = u16::MAX as usize;

static MAX_BLOCK_ITEM_TEXTURES: usize = u16::MAX as usize;

static MAX_MAP_TEXTURES: usize = u16::MAX as usize;

/// The main game structure
pub struct Game {
    pub player: Player,
    tile_map: TileMapManager,
    world_generator: WorldGenerator,

    // if a lot of unique ui elements are added, this could be abstracted into its own ui manager struct
    pub player_ui_manager: PlayerUiManager,  // storing this external to player since it can't be saved (and really doesn't need to be)

    pub(crate) entity_manager: EntityManager,

    pub(crate) random_state: rand::rngs::ThreadRng,

    mini_map_textures: Vec<[u32; 16]>,
}

impl Game {
    pub fn save(&self, path_prefix: &str, version: &str, logs: &mut Logs) -> Result<(), std::io::Error> {
        let start = std::time::Instant::now();
        let config = bincode::config::standard();
        let encoded: Vec<u8> = bincode::encode_to_vec(&self.player, config).unwrap();
        std::fs::write(format!("{}/game_version_{}/player/player.bin", path_prefix, version), &encoded)?;  // just dump to file

        let encoded: Vec<u8> = bincode::encode_to_vec(&self.tile_map, config).unwrap();
        std::fs::write(format!("{}/game_version_{}/world_save/tile_map.bin", path_prefix, version), &encoded)?;  // just dump to file

        let encoded: Vec<u8> = bincode::encode_to_vec(&self.world_generator, config).unwrap();
        std::fs::write(format!("{}/game_version_{}/world_save/world_generator.bin", path_prefix, version), &encoded)?;  // just dump to file

        let encoded: Vec<u8> = bincode::encode_to_vec(&self.entity_manager, config).unwrap();
        std::fs::write(format!("{}/game_version_{}/world_save/entities/entity.bin", path_prefix, version), &encoded)?;  // just dump to file

        logs.push(Log {
            message: format!("Saved game in {} seconds", start.elapsed().as_secs_f64()),
            level: crate::logging::logging::LoggingError::Info
        }, 62, LogType::Information);

        Ok(())
    }

    // the version parameter should hopefully make it easier to update old saves into newer versions by targeting them specifically
    fn file_loader<T: bincode::Decode<()>>(path: &str) -> Result<T, GameError> {
        let config = bincode::config::standard();
        let data = std::fs::read(path).map_err(|e| GameError {
            message: format!("Failed to deserialize file: {}\nError: {:?}", path, e),
            severity: Severity::Fatal,
        })?;
        let (decoded, _len): (T, usize) = bincode::decode_from_slice(&data, config).map_err(|e| GameError {
            message: format!("Failed to deserialize file: {}\nError: {:?}", path, e),
            severity: Severity::Fatal,
        })?;
        Ok(decoded)
    }

    // the version parameter should hopefully make it easier to update old saves into newer versions by targeting them specifically
    pub fn from_save(logs: &mut Logs, path_prefix: &str, version: &str, font_atlas: Rc<Vec<[u32; 256]>>) -> Result<Self, GameError> {
        let player: Player = Self::file_loader(&format!("{}/game_version_{}/player/player.bin", path_prefix, version))?;
        let tile_map: TileMapManager = Self::file_loader(&format!("{}/game_version_{}/world_save/tile_map.bin", path_prefix, version))?;
        let world_generator: WorldGenerator = Self::file_loader(&format!("{}/game_version_{}/world_save/world_generator.bin", path_prefix, version))?;
        let entity: EntityManager = Self::file_loader(&format!("{}/game_version_{}/world_save/entities/entity.bin", path_prefix, version))?;
        Ok(Game {
            player,
            tile_map,
            world_generator,
            player_ui_manager: PlayerUiManager::new({
                let mut total_textures_loaded = 0;
                let textures = get_texture_atlas::<MAX_ITEM_TEXTURES, 256>("textures/items/", (16, 16), vec![[0u32; 256]; MAX_ITEM_TEXTURES], &mut total_textures_loaded)
                    .map_err(|e| GameError {
                        message: format!("[Game Startup Error] Failed to load textures for items: {:?}", e),
                        severity: Severity::Fatal
                    })?;
                logs.push(Log {
                    message: format!("Loaded {} item textures.", total_textures_loaded - 1),
                    level: crate::logging::logging::LoggingError::Info,
                }, 11, LogType::Information);
                textures
            }, logs, font_atlas).map_err(|e| GameError {
                message: e.details,
                severity: Severity::Fatal
            })?,
            entity_manager: entity,
            random_state: rand::rng(),
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
                textures.map_err(|e| GameError {
                    message: format!("{:?}", e),
                    severity: Severity::Fatal
                })?
            },
        })
    }

    pub fn new(logs: &mut Logs, font_atlas: Rc<Vec<[u32; 256]>>) -> Result<Self, GameError> {
        let world_generator = WorldGenerator::new();
        let mut tile_map_manager = TileMapManager::new();
        // todo! temporary for now; eventually a world creation menue will be added
        tile_map_manager.replace_tile_map(
            Dimension::Overworld,
            TileMap::new(4095, 1024, Some(&world_generator), logs)?
        );
        tile_map_manager.get_current_map(Dimension::Overworld)
            .ok_or_else(|| GameError { message: String::from("Failed to get current map"), severity: Severity::Fatal })?
            .add_entity_light(String::from("Player"), (0.0, 0.0), (225, 225, 128, 0.65));
        Ok(Game {
            player: Player::new(),
            tile_map: tile_map_manager,
            world_generator: world_generator,
            player_ui_manager: PlayerUiManager::new({
                let mut total_textures_loaded = 0;
                let textures = get_texture_atlas::<MAX_ITEM_TEXTURES, 256>("textures/items/", (16, 16), vec![[0u32; 256]; MAX_ITEM_TEXTURES], &mut total_textures_loaded)
                    .map_err(|e| GameError {
                        message: format!("[Game Startup Error] Failed to load textures for items: {:?}", e),
                        severity: Severity::Fatal
                    })?;
                logs.push(Log {
                    message: format!("Loaded {} item textures.", total_textures_loaded - 1),
                    level: crate::logging::logging::LoggingError::Info,
                }, 10, LogType::Information);
                textures
            }, logs, font_atlas).map_err(|e| GameError {
                message: e.details,
                severity: Severity::Fatal
            })?,
            entity_manager: EntityManager::new(),
            random_state: rand::rng(),
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
                textures.map_err(|e| GameError {
                    message: format!("{:?}", e),
                    severity: Severity::Fatal
                })?
            },
        })
    }

    pub fn update_key_events(
        &mut self, timer: &crate::core::timer::Timer,
        event_handler: &crate::core::event_handling::event_handler::EventHandler,
        screen_size: (u32, u32),
        logs: &mut Logs,
    ) -> Result<(), GameError> {
        if let Some(tile_map) = self.tile_map.get_current_map(Dimension::Overworld) {
            if tile_map.entity_lights.len() > 256 {
                logs.push(Log {
                    message: format!("[Memory Warning] Total dynamic lights has exceeded a reasonable count. Current count: {}", tile_map.entity_lights.len()),
                    level: crate::logging::logging::LoggingError::Warning,
                }, 9, LogType::Memory);
            }
            self.player.update_key_events(
                timer,
                event_handler,
                tile_map,
                screen_size,
                &mut self.player_ui_manager,
                &mut self.entity_manager,
                &mut self.random_state,
            )?;
        }
        
        // doing some checks and possibly logging anything abnormal or that could be logged
        if self.entity_manager.get_entity_count() > MAX_ENTITIES {
            logs.push(Log {
                message: format!("[Memory Warning] Total entity count is higher than expected: {}", self.entity_manager.get_entity_count()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 1, LogType::Memory);
        }

        if event_handler.keys_held.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.keys_held.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 2, LogType::Memory)
        }
        if event_handler.keys_pressed.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.keys_pressed.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 3, LogType::Memory)
        }
        if event_handler.keys_released.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.keys_released.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 4, LogType::Memory)
        }
        if event_handler.mods_held.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.mods_held.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 5, LogType::Memory)
        }
        if event_handler.mods_pressed.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.mods_pressed.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 6, LogType::Memory)
        }
        if event_handler.mods_released.len() > 128 {
            logs.push(Log {
                message: format!("[Memory Warning] Total Keyboard Events exceeded the predetermined warning threshold. Current count: {}", event_handler.mods_released.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 7, LogType::Memory)
        }
        if self.player_ui_manager.ui_elements.len() > 16 {
            logs.push(Log {
                message: format!("[Memory Warning] Total UI elements allocated for the player exceeded the predetermined warning threshold. Current count: {}", self.player_ui_manager.ui_elements.len()),
                level: crate::logging::logging::LoggingError::Warning,
            }, 8, LogType::Memory)
        }

        Ok(())
    }

    pub fn get_tilemap_manager(&mut self) -> &mut TileMapManager {
        &mut self.tile_map
    }

    pub fn get_tilemap_manager_ref(&self) -> &TileMapManager {
        &self.tile_map
    }

    pub fn render_ui(&mut self, buffer: &mut [u8], window_size: (u32, u32), pitch: usize) -> Result<(), crate::core::rendering::ui::UiError> {
        // rendering any ui related to the player
        self.player.render_ui(buffer, window_size, &mut self.player_ui_manager, pitch)?;
        
        // rendering the mini map
        let camera_x = self.player.camera.x;
        let camera_y = self.player.camera.y;
        if let Some(map) = self.tile_map.get_current_map(Dimension::Overworld) {
            map.mini_map.camera_transform.x = camera_x;
            map.mini_map.camera_transform.y = camera_y;
            map.mini_map.render(
                &map.tiles,buffer, (350, 200), (window_size.0 as usize - 375, 25), pitch, &self.mini_map_textures
            );
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GameError {
    pub message: String,
    pub severity: Severity,
}

impl From<GameError> for String {
    fn from(error: GameError) -> Self {
        format!("[Game Error of Severity: {:?}] {}", error.severity, error.message)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Fatal,
}

