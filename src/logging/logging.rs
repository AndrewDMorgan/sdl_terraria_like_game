
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

#[derive(serde::Serialize, serde::Deserialize)]
struct LogsSerializable {
    pub logs: Vec<Log>,
    pub updated: bool,
}

// wraps the Log into a vector, but alliased to allow serialization
pub struct Logs(LogsSerializable, crossbeam::channel::Receiver<bool>);

impl Logs {
    pub fn new(channel: crossbeam::channel::Receiver<bool>) -> Self {
        Self(LogsSerializable { logs: Vec::new(), updated: false }, channel)
    }

    /// Checks if the logs were updated since the last check
    pub fn was_updated(&mut self) -> bool {
        if self.0.updated {
            self.0.updated = false;
            true
        } else {
            false
        }
    }

    /// Adds a new log entry and marks the logs as updated
    pub fn push(&mut self, log: Log) {
        self.0.logs.push(log);
        self.0.updated = true;
    }

    /// Saves the current logs to the logs file in JSON format
    pub fn save(&self) -> Result<(), String> {
        let log_json = serde_json::to_string_pretty(&self.0)
            .map_err(|e| e.to_string())?;
        std::fs::write(LOGS_PATH, log_json)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Drop for Logs {
    fn drop(&mut self) {
        // the main execution ended before being able to signal an end, meaning a fatal and non-unwinding error happened
        // this could be something like an array access out of bounds, or some other non-unwinding error, as they don't return a result or information
        // to the std catch_unwind. This at least should provide the information that something of that nature happened, as, unfortunately,
        // other information isn't sent when this happens, and I don't think it's possible to get further info on the exact error
        if self.1.try_recv().is_err() {
            self.0.logs.push(Log {
                message: format!("[Uncaught] Fatal Error: error didn't unwind, and wasn't caught"), level: LoggingError::Error
            });
        }

        if self.0.updated { self.save().unwrap(); }
        println!("{:?}", self.0.logs.iter().filter(|log| {
            match log.level {
                LoggingError::Error => true,
                _ => false,
            }
        }).collect::<Vec<_>>());
    }
}

