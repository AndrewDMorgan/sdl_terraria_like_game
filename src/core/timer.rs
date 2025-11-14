
/// A simple timer struct for tracking frame times and delta times.
pub struct Timer {
    start: std::time::Instant,
    frame: u128,
    frame_start: f64,
    pub delta_time: f64,
    accumulative_time: f64,
}

impl Timer {
    /// Creates a new Timer instance and initializes the start time.
    pub fn new() -> Self {
        Timer {
            start: std::time::Instant::now(),
            frame: 0,
            frame_start: 0.0,
            delta_time: 0.0,
            accumulative_time: 0.0,
        }
    }

    /// Starts a new frame, updating the frame start time.
    pub fn start_new_frame(&mut self) {
        self.frame_start = self.start.elapsed().as_secs_f64();
    }

    /// Returns the duration of the current frame.
    pub fn elapsed_frame(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.start.elapsed().as_secs_f64() - self.frame_start)
    }

    /// Updates the frame data, including the delta time and frame count.
    pub fn update_frame_data(&mut self) {
        let current_time = self.start.elapsed().as_secs_f64();
        self.delta_time = (current_time - self.frame_start).min(0.1);
        self.frame += 1;

        self.accumulative_time += self.delta_time;
    }
}

impl Drop for Timer {
    // todo! temporary
    fn drop(&mut self) {
        println!("Average runtime: {:.3} fps", 1.0 / (self.accumulative_time / self.frame as f64));
    }
}

