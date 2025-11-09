use crate::logging;


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

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
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
    pub logs: Vec<(Log, String)>,
    pub updated: bool,
}

// wraps the Log into a vector, but alliased to allow serialization
pub struct Logs {
    logs: LogsSerializable,
    receiver: crossbeam::channel::Receiver<bool>,
    pub message_ids: std::collections::HashMap<usize, std::time::Instant>,
    logging_level: Logging,
}

impl Logs {
    pub fn new(receiver: crossbeam::channel::Receiver<bool>, logging_level: Logging) -> Self {
        Self {
            logs: LogsSerializable { logs: Vec::new(), updated: false },
            receiver,
            message_ids: std::collections::HashMap::new(),
            logging_level,
        }
    }

    /// Checks if the logs were updated since the last check
    pub fn was_updated(&mut self) -> bool {
        if self.logs.updated {
            self.logs.updated = false;
            true
        } else {
            false
        }
    }

    /// Adds a new log entry and marks the logs as updated
    pub fn push(&mut self, log: Log, message_id: usize, level: LogType) {
        match level {
            LogType::Error => {
                if !matches!(self.logging_level, Logging::ErrorOnly | Logging::Everything) { return; }
            },
            LogType::Information => {
                if !matches!(self.logging_level, Logging::Everything) { return; }
            },
            LogType::Memory => {
                if !matches!(self.logging_level, Logging::MemoryOnly | Logging::Everything | Logging::WarningOnly) { return; }
            },
            LogType::Performance => {
                if !matches!(self.logging_level, Logging::PerformanceOnly | Logging::Everything | Logging::WarningOnly) { return; }
            },
            LogType::Warning => {
                if !matches!(self.logging_level, Logging::WarningOnly | Logging::Everything) { return; }
            },
        }
        // making sure the logs don't get spammed from repeated calls
        if log.level != LoggingError::Error {  // errors should always be logged, warnings and info should be rate-limited
            if let Some(time) = self.message_ids.get(&message_id) {
                if time.elapsed().as_millis() < 250 { return; }
            }
        }
        self.message_ids.insert(message_id, std::time::Instant::now());
        self.logs.logs.push((log, format!("{:?}", datetime::LocalDateTime::now())));
        self.logs.updated = true;
    }

    /// Saves the current logs to the logs file in JSON format
    pub fn save(&self) -> Result<(), String> {
        let log_json = serde_json::to_string_pretty(&self.logs)
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
        if self.receiver.try_recv().is_err() {
            self.logs.logs.push((Log {
                message: format!("[Uncaught] Fatal Error: error didn't unwind, and wasn't caught"), level: LoggingError::Error
            }, format!("{:?}", datetime::LocalDateTime::now())));
        }

        if self.logs.updated { self.save().unwrap(); }
        println!("{:?}", self.logs.logs.iter().filter(|log| {
            match log.0.level {
                LoggingError::Error => true,
                _ => false,
            }
        }).collect::<Vec<_>>());
    }
}

pub enum LogType {
    Memory,
    Performance,
    Information,
    Error,
    Warning
}

