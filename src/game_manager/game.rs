
use crate::game_manager::entities::player::player::*;
use crate::game_manager::world::tile_map::*;
use crate::game_manager::world::world_gen::*;

/// The main game structure
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub player: Player,
    tile_map: TileMapManager,
    world_generator: WorldGenerator,
}

impl Game {
    pub fn new() -> Self {
        let world_generator = WorldGenerator::new();
        let mut tile_map_manager = TileMapManager::new();
        // todo! temporary for now; eventually a world creation menue will be added
        tile_map_manager.replace_tile_map(
            Dimension::Overworld,
            TileMap::new(4095, 256, Some(&world_generator))
        );
        Game {
            player: Player::new(),
            tile_map: tile_map_manager,
            world_generator: world_generator,
        }
    }

    pub fn update_key_events(
        &mut self, timer: &crate::core::timer::Timer,
        event_handler: &crate::core::event_handling::event_handler::EventHandler,
        screen_size: (u32, u32)
    ) {
        if let Some(tile_map) = self.tile_map.get_current_map(Dimension::Overworld) {
            self.player.update_key_events(timer, event_handler, tile_map, screen_size);
        }
    }

    pub fn get_tilemap_manager(&mut self) -> &mut TileMapManager {
        &mut self.tile_map
    }

    pub fn get_tilemap_manager_ref(&self) -> &TileMapManager {
        &self.tile_map
    }
}

#[derive(Debug)]
pub struct GameError {
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Fatal,
}

