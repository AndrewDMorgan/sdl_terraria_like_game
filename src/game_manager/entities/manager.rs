use crate::game_manager::entities::player::player::CameraTransform;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EntityManager {
    drops: Vec<(ItemDrop, u32, u32)>,
}

impl EntityManager {
    pub fn new() -> Self {
        EntityManager {
            drops: Vec::new(),
        }
    }
    
    pub fn new_drop(&mut self, drop: ItemDrop, pos_x: u32, pos_y: u32) {
        self.drops.push((drop, pos_x, pos_y));
    }

    pub fn get_render(&self, camera: &CameraTransform, screen_width: (u32, u32)) -> Vec<(u32, u16, i16, i16, u16, u32)> {
        let mut render_data = vec![];
        let edge_x = (screen_width.0 as f32) * 0.5 * camera.zoom;
        let edge_y = (screen_width.1 as f32) * 0.5 * camera.zoom;

        // the gpu pipeline actually does work a lot better, even if the algerithm is slower on paper
        for drop in &self.drops {
            match &drop.0 {
                ItemDrop::Tile(tile_texture_id) => {
                    let position = (drop.1 as f32 - camera.x, drop.2 as f32 - camera.y);
                    if position.0 < -edge_x - 4.0 || position.1 < -edge_y - 4.0 || position.0 > edge_x || position.1 > edge_y {
                        continue;
                    }
                    render_data.push((*tile_texture_id as u32, 0, (position.0 * 100.0) as i16, (position.1 * 100.0) as i16, 0, 0));
                },
            }
        } render_data
    }

    pub fn get_entity_count(&self) -> usize {
        self.drops.len()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum ItemDrop {
    Tile (u32),
}

