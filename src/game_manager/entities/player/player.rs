
/// The player entity module
pub struct Player {
    pub camera: CameraTransform,
}

impl Player {
    pub fn new() -> Self {
        Player {
            camera: CameraTransform {
                x: 0.0,
                y: 135.0 * 8.0,
                zoom: 0.2,
            },
        }
    }
}

pub struct CameraTransform {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

