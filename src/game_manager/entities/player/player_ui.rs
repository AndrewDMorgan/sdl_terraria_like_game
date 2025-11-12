use crate::{core::rendering::ui::UiElement,
            game_manager::entities::player::{inventory::generate_player_hotbar_ui_element, player::PlayerData}};
use std::rc::Rc;

#[derive(Default)]
pub struct PlayerUiManager {
    pub ui_elements: Vec<UiElement<PlayerData>>,
    // this is an rc so that it can be passed into a ui renderer closure without issue (anyway, it's constant and doesn't change after initialization)
    pub item_textures: Rc<Vec<[u32; 256]>>,
    pub text_character_atlas: Rc<Vec<[u32; 256]>>,
}

impl PlayerUiManager {
    pub fn new(item_textures: Vec<[u32; 256]>, font_atlas: Rc<Vec<[u32; 256]>>) -> Result<Self, crate::textures::textures::TextureError> {
        let item_textures = Rc::new(item_textures);
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

