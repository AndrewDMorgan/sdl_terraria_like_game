use crate::logging::logging::LoggingError;


#[derive(Debug)]
pub struct UiError {
    pub(crate) message: String,
    pub(crate) severity: LoggingError,
}

impl From<UiError> for String {
    fn from(error: UiError) -> Self {
        format!("{:?}: {}", error.severity, error.message)
    }
}

pub struct UiElement<T> {
    position: (usize, usize),
    size: (usize, usize),
    renderer: Box<dyn Fn(&mut [u8], ((u32, u32), usize), &mut T, ((usize, usize), (usize, usize))) -> Result<(), UiError>>,
    pub identifier: String,
}

impl<T> UiElement<T> {
    pub fn new(identifier: String, position: (usize, usize), size: (usize, usize), renderer: Box<dyn Fn(&mut [u8], ((u32, u32), usize), &mut T, ((usize, usize), (usize, usize))) -> Result<(), UiError>>) -> Self {
        Self {
            position,
            size,
            renderer,
            identifier
        }
    }

    pub fn render(&self, buffer: &mut [u8], buffer_size: (u32, u32), pitch: usize, state: &mut T) -> Result<(), UiError> {
        (self.renderer)(buffer, (buffer_size, pitch), state, (self.position, self.size))
    }
}

