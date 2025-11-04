use crate::{textures::animation::Animator, utill::Union};


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Hitbox {
    pub offset: (f64, f64),
    pub size: (f64, f64),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Sprite<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    size: (f64, f64),
    offset: (f64, f64),
    sprite_layout: Vec<usize>,  // the shape fo the srpite to be rendered
    hit_box: Hitbox,
    texture: Union<u32, Animator<T>>
}

impl<T> Sprite<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    pub fn new_static(size: (f64, f64), offset: (f64, f64), sprite_layout: Vec<usize>, hit_box: Hitbox, texture: u32) -> Self {
        Sprite {
            size,
            offset,
            sprite_layout,
            hit_box,
            texture: Union::A(texture),
        }
    }

    pub fn new_animated(size: (f64, f64), offset: (f64, f64), sprite_layout: Vec<usize>, hit_box: Hitbox, texture: Animator<T>) -> Self {
        Sprite {
            size,
            offset,
            sprite_layout,
            hit_box,
            texture: Union::B(texture),
        }
    }

    pub fn get_size(&self) -> (f64, f64) {
        self.size
    }

    pub fn get_offset(&self) -> (f64, f64) {
        self.offset
    }

    pub fn get_hitbox(&self) -> &Hitbox {
        &self.hit_box
    }

    pub fn get_texture(&self) -> u32 {
        match self.texture {
            Union::A(ref tex) => *tex,
            Union::B(ref animator) => animator.get_current_sprite(),
        }
    }

    pub fn update_frame(&mut self, delta_time: f64) {
        if let Union::B(ref mut animator) = self.texture {
            animator.cycle_animation(delta_time);
        }
    }

    pub fn set_animation(&mut self, animation: T) {
        if let Union::B(ref mut animator) = self.texture {
            animator.set_animation(animation);
        }
    }
}


