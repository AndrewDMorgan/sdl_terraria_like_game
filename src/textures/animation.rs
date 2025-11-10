
#[derive(bincode::Encode, bincode::Decode)]
pub struct Animator<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    sprites: Vec<Vec<u32>>,
    durations: Vec<f64>,
    current_animation: T,
    current_frame: usize,
    timer: f64,
}

impl<T> Animator<T>
    where T: Into<u8> + From<u8> + Default + Copy
{
    pub fn new(sprites: Vec<Vec<u32>>, durations: Vec<f64>) -> Self {
        Animator {
            sprites,
            durations,
            current_animation: T::default(),
            current_frame: 0,
            timer: 0.0,
        }
    }

    pub fn get_current_sprite(&self) -> u32 {
        self.sprites[self.current_animation.into() as usize][self.current_frame]
    }

    pub fn set_animation(&mut self, animation: T) {
        if self.current_animation.into() != animation.into() {
            self.current_animation = animation;
            self.timer = 0.0;
            self.current_frame = 0;
        }
    }

    pub fn cycle_animation(&mut self, delta_time: f64) {
        self.timer += delta_time;
        let animation_index = self.current_animation.into() as usize;
        if self.timer >= self.durations[animation_index] {
            self.timer = 0.0;
            self.current_frame += 1;
            if self.current_frame >= self.sprites[animation_index].len() {
                self.current_frame = 0;
            }
        }
    }
}


