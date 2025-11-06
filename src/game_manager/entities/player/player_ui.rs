use crate::{core::rendering::ui::UiElement, game_manager::entities::player::player::PlayerData};

#[derive(Default)]
pub struct PlayerUiManager {
    ui_elements: Vec<UiElement<PlayerData>>,
}

impl PlayerUiManager {
    pub fn render_ui(&self, buffer: &mut [u8], buffer_size: (u32, u32), player_data: &mut PlayerData) -> Result<(), crate::core::rendering::ui::UiError> {
        for element in &self.ui_elements {
            element.render(buffer, buffer_size, player_data)?;
        } Ok(())
    }
}

