use crate::game_manager::entities::player::{player::*, player_ui::PlayerUiManager};
use crate::game_manager::world::{world_gen::*, tile_map::*};
use crate::textures::textures::get_texture_atlas;
use crate::logging::logging::{Log, Logs};

// the maximum item textures (in this context, this can safely be set to any value >= to the total texture count as no gpu buffers rely upon a static size for this)
static MAX_ITEM_TEXTURES: usize = u16::MAX as usize;

/// The main game structure
pub struct Game {
    pub player: Player,
    tile_map: TileMapManager,
    world_generator: WorldGenerator,

    // if a lot of unique ui elements are added, this could be abstracted into its own ui manager struct
    player_ui_manager: PlayerUiManager,  // storing this external to player since it can't be saved (and really doesn't need to be)
}

impl Game {
    pub fn save(&self, path_prefix: &str, version: &str) -> Result<(), serde_json::Error> {
        let file = std::fs::File::create(&format!("{}/world_save_version_{}/player/player.json", path_prefix, version)).unwrap();
        serde_json::to_writer(file, &self.player)?;

        let file = std::fs::File::create(&format!("{}/world_save_version_{}/world_save/tile_map.json", path_prefix, version)).unwrap();
        serde_json::to_writer(file, &self.tile_map)?;

        let file = std::fs::File::create(&format!("{}/world_save_version_{}/world_save/world_generator.json", path_prefix, version)).unwrap();
        serde_json::to_writer(file, &self.world_generator)?;

        Ok(())
    }

    // the version parameter should hopefully make it easier to update old saves into newer versions by targeting them specifically
    fn file_loader<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, GameError> {
        match std::fs::File::open(path) {
            Ok(data) => {
                let reader = std::io::BufReader::new(data);
                Ok(serde_json::from_reader::<_, T>(reader).map_err(|e| {
                    GameError {
                        message: format!("Failed to deserialize file: {}\nError: {:?}", path, e),
                        severity: Severity::Fatal,
                    }
                })?)
            },
            _ => Err(GameError {
                message: format!("Failed to open file: {}", path),
                severity: Severity::Fatal,
            }),
        }
    }

    // the version parameter should hopefully make it easier to update old saves into newer versions by targeting them specifically
    pub fn from_save(logs: &mut Logs, path_prefix: &str, version: &str) -> Result<Self, GameError> {
        let player = Self::file_loader(&format!("{}/world_save_version_{}/player/player.json", path_prefix, version))?;
        let tile_map = Self::file_loader(&format!("{}/world_save_version_{}/world_save/tile_map.json", path_prefix, version))?;
        let world_generator = Self::file_loader(&format!("{}/world_save_version_{}/world_save/world_generator.json", path_prefix, version))?;
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
                });
                textures
            }, logs).map_err(|e| GameError {
                message: e.details,
                severity: Severity::Fatal
            })?,
        })
    }

    pub fn new(logs: &mut Logs) -> Result<Self, GameError> {
        let world_generator = WorldGenerator::new();
        let mut tile_map_manager = TileMapManager::new();
        // todo! temporary for now; eventually a world creation menue will be added
        tile_map_manager.replace_tile_map(
            Dimension::Overworld,
            TileMap::new(4095, 256, Some(&world_generator))?
        );
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
                });
                textures
            }, logs).map_err(|e| GameError {
                message: e.details,
                severity: Severity::Fatal
            })?,
        })
    }

    pub fn update_key_events(
        &mut self, timer: &crate::core::timer::Timer,
        event_handler: &crate::core::event_handling::event_handler::EventHandler,
        screen_size: (u32, u32),
    ) -> Result<(), GameError> {
        if let Some(tile_map) = self.tile_map.get_current_map(Dimension::Overworld) {
            self.player.update_key_events(timer, event_handler, tile_map, screen_size, &mut self.player_ui_manager)?;
        } Ok(())
    }

    pub fn get_tilemap_manager(&mut self) -> &mut TileMapManager {
        &mut self.tile_map
    }

    pub fn get_tilemap_manager_ref(&self) -> &TileMapManager {
        &self.tile_map
    }

    pub fn render_ui(&mut self, buffer: &mut [u8], buffer_size: (u32, u32), pitch: usize) -> Result<(), crate::core::rendering::ui::UiError> {
        // rendering any ui related to the player
        self.player.render_ui(buffer, buffer_size, &mut self.player_ui_manager, pitch)?;

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

