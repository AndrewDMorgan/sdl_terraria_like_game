
/// Module for handling events in the application
pub struct EventHandler {
    //
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            //
        }
    }

    /// Handles incoming events and returns a status indicating the result
    pub fn handle_events<I>(&mut self, events: I) -> Status
    where
        I: Iterator<Item = sdl2::event::Event>,
    {
        for event in events {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    return Status::Quit;
                }
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

