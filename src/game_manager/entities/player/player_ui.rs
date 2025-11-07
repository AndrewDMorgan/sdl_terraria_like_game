use crate::{core::rendering::ui::UiElement,
            game_manager::entities::player::{inventory::generate_player_hotbar_ui_element, player::PlayerData},
            logging::logging::Log, textures::textures::get_texture_atlas};
use std::rc::Rc;

static MAX_FONT_CHARACTERS: usize = u16::MAX as usize;

#[derive(Default)]
pub struct PlayerUiManager {
    ui_elements: Vec<UiElement<PlayerData>>,
    // this is an rc so that it can be passed into a ui renderer closure without issue (anyway, it's constant and doesn't change after initialization)
    item_textures: Rc<Vec<[u32; 256]>>,
    text_character_atlas: Rc<Vec<[u32; 256]>>,
}

impl PlayerUiManager {
    pub fn new(item_textures: Vec<[u32; 256]>, logs: &mut crate::logging::logging::Logs) -> Result<Self, crate::textures::textures::TextureError> {
        let item_textures = Rc::new(item_textures);
        let font_atlas = Rc::new({
            let mut total_textures_loaded = 0;
            let atlas = get_texture_atlas::<MAX_FONT_CHARACTERS, 256>("textures/fonts/user_default/", (16, 16), vec![[0u32; 256]; MAX_FONT_CHARACTERS], &mut total_textures_loaded)?;
            logs.push(Log {
                message: format!("Loaded {} font characters for the player ui rendering.", total_textures_loaded - 1),
                level: crate::logging::logging::LoggingError::Info
            });
            atlas
        });
        Ok(Self {
            ui_elements: vec![
                // creating the hotbar
                generate_player_hotbar_ui_element(item_textures.clone(), font_atlas.clone()),
            ],
            item_textures: item_textures,
            text_character_atlas: font_atlas,
        })
    }

    pub fn render_ui(&self, buffer: &mut [u8], buffer_size: (u32, u32), player_data: &mut PlayerData, pitch: usize) -> Result<(), crate::core::rendering::ui::UiError> {
        for element in &self.ui_elements {
            element.render(buffer, buffer_size, pitch, player_data)?;
        } Ok(())
    }
}

