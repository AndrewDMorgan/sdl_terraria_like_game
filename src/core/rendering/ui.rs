use crate::logging::logging::LoggingError;


#[derive(Debug)]
pub struct UiError {
    message: String,
    severity: LoggingError,
}

impl From<UiError> for String {
    fn from(error: UiError) -> Self {
        error.message
    }
}

pub struct UiElement<T> {
    position: (usize, usize),
    size: (usize, usize),
    renderer: Box<dyn Fn(&mut [u8], (u32, u32), &mut T) -> Result<(), UiError>>,
}

impl<T> UiElement<T> {
    pub fn new(position: (usize, usize), size: (usize, usize), renderer: Box<dyn Fn(&mut [u8], (u32, u32), &mut T) -> Result<(), UiError>>) -> Self {
        Self {
            position,
            size,
            renderer,
        }
    }

    pub fn render(&self, buffer: &mut [u8], buffer_size: (u32, u32), state: &mut T) -> Result<(), UiError> {
        (self.renderer)(buffer, buffer_size, state)
    }
}

