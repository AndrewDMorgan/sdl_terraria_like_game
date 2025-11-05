
/// The path to the logs file
pub static LOGS_PATH: &str = "Logs/logs.json";

/// The threshold for logging performance-related events
pub static PERFORMANCE_LOG_THRESHOLD: f64 = 1.0 / 60.0; // 16.67 ms or 60 fps

/// Defines the logging level for the application
pub enum Logging {
    Everything,
    ErrorOnly,
    Nothing,
    WarningOnly,
    MemoryOnly,
    PerformanceOnly,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum LoggingError {
    Warning,
    Error,
    Info,
}

// a basic logging function to make reading errors slightly easier
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub message: String,
    pub level: LoggingError,
}

// wraps the Log into a vector, but alliased to allow serialization
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Logs(pub Vec<Log>, pub bool);
impl Logs {
    /// Checks if the logs were updated since the last check
    pub fn was_updated(&mut self) -> bool {
        if self.1 {
            self.1 = false;
            true
        } else {
            false
        }
    }

    /// Adds a new log entry and marks the logs as updated
    pub fn push(&mut self, log: Log) {
        self.0.push(log);
        self.1 = true;
    }

    /// Saves the current logs to the logs file in JSON format
    pub fn save(&self) -> Result<(), String> {
        let log_json = serde_json::to_string_pretty(self)
            .map_err(|e| e.to_string())?;
        std::fs::write(LOGS_PATH, log_json)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Drop for Logs {
    fn drop(&mut self) {
        if self.1 { self.save().unwrap(); }
        println!("{:?}", self.0.iter().filter(|log| {
            match log.level {
                LoggingError::Error => true,
                _ => false,
            }
        }).collect::<Vec<_>>());
    }
}

