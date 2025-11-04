use crate::{core::timer::Timer, game_manager::game::Game};

pub enum ButtonState {
    Pressed,
    Held,
    Released,
    Idle,
}

pub struct Mouse {
    pub left: ButtonState,
    pub right: ButtonState,
    pub position: (u32, u32),
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            left: ButtonState::Idle,
            right: ButtonState::Idle,
            position: (0, 0),
        }
    }
}

/// Module for handling events in the application
pub struct EventHandler {
    pub keys_pressed: Vec<sdl2::keyboard::Keycode>,
    pub keys_released: Vec<sdl2::keyboard::Keycode>,
    pub keys_held: Vec<sdl2::keyboard::Keycode>,
    pub mouse: Mouse,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            keys_pressed: Vec::new(),
            keys_released: Vec::new(),
            keys_held: Vec::new(),
            mouse: Mouse::new(),
        }
    }

    /// Handles incoming events and returns a status indicating the result
    pub fn handle_events(&mut self, event_pump: &mut sdl2::EventPump, game: &mut Option<&mut Game>, timer: &Timer) -> Status {
        let mouse_state = sdl2::mouse::MouseState::new(event_pump);
        self.mouse.position = (mouse_state.x() as u32, mouse_state.y() as u32);
        self.mouse.right = match mouse_state.right() {
            true => {
                if let ButtonState::Pressed | ButtonState::Held = self.mouse.right {
                    ButtonState::Held
                } else {
                    ButtonState::Pressed
                }
            },
            false => {
                if let ButtonState::Released | ButtonState::Idle = self.mouse.right {
                    ButtonState::Idle
                } else {
                    ButtonState::Released
                }
            },
        };
        self.mouse.left = match mouse_state.left() {
            true => {
                if let ButtonState::Pressed | ButtonState::Held = self.mouse.left {
                    ButtonState::Held
                } else {
                    ButtonState::Pressed
                }
            },
            false => {
                if let ButtonState::Released | ButtonState::Idle = self.mouse.left {
                    ButtonState::Idle
                } else {
                    ButtonState::Released
                }
            },
        };

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    return Status::Quit;
                },
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                    if !self.keys_pressed.contains(&keycode) {
                        self.keys_pressed.push(keycode);
                    }
                    if !self.keys_held.contains(&keycode) {
                        self.keys_held.push(keycode);
                    }
                },
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    if !self.keys_released.contains(&keycode) {
                        self.keys_released.push(keycode);
                    }
                    self.keys_held.retain(|&k| k != keycode);
                },
                _ => {}
            }
        }
        Status::Continue
    }
}

/// Status enum to indicate the result of event handling
pub enum Status {
    Error(String, u8),  // higher means worse; u8 max means fatal
    Quit,
    Continue,
}

