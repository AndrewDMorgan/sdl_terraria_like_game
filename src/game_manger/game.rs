
use crate::game_manger::entities::player::player::*;

/// The main game structure
pub struct Game {
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        Game {
            player: Player::new(),
        }
    }
}

