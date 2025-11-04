use crate::textures::sprite::Sprite;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Entity<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    pub sprite: Sprite<T>,
    pub position: (f32, f32),
}

impl<T> Entity<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    pub fn new(sprite: Sprite<T>, position: (f32, f32)) -> Self {
        Entity {
            sprite,
            position,
        }
    }
}

