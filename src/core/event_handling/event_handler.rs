use crate::{core::timer::Timer, game_manager::game::Game};


/// Module for handling events in the application
pub struct EventHandler {
    pub keys_pressed: Vec<sdl2::keyboard::Keycode>,
    pub keys_released: Vec<sdl2::keyboard::Keycode>,
    pub keys_held: Vec<sdl2::keyboard::Keycode>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            keys_pressed: Vec::new(),
            keys_released: Vec::new(),
            keys_held: Vec::new(),
        }
    }

    /// Handles incoming events and returns a status indicating the result
    pub fn handle_events<I>(&mut self, events: I, game: &mut Option<&mut Game>, timer: &Timer) -> Status
    where
        I: Iterator<Item = sdl2::event::Event>,
    {
        for event in events {
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

