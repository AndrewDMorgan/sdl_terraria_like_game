/*
  To compile:

there's a messy thing going on with homebrew (causing sdl2 to fail to link to the arms version) so do:

echo 'export LIBRARY_PATH="/opt/homebrew/lib"' >> ~/.zshrc
echo 'export C_INCLUDE_PATH="/opt/homebrew/include"' >> ~/.zshrc
source ~/.zshrc
export LIBRARY_PATH="/opt/homebrew/lib"
export C_INCLUDE_PATH="/opt/homebrew/include"

this seems to work, idk which parts are necessary, so just run it in every new terminal session before compiling
this should be a problem localized only to my mac, but who knows

well, ig it maybe only needs to be run after restart? Or did this just fix it? idk, but it seems to be working now

*/

use crate::logging::logging::{Log, LoggingError, Logs};

mod shaders;
mod game_manager;
mod textures;
mod logging;
mod core;
mod utill;

fn main() {
    let (concluded_sender, concluded_receiver) = crossbeam::channel::bounded(1);
    let mut logs = Logs::new(concluded_receiver);

    // using a raw pointer, because, if a non-unwound error is thrown, it crashes, but a mutex lock isn't dropped (even though the thread crashed)
    // so it should be fine using it here as by the time the catch_unwind is done, it should be no longer in use
    // this just guarantees that the logs can be accessed afterwards
    let logs_ptr = &mut logs as *mut Logs;
    let result = std::panic::catch_unwind(|| {
        let logs = unsafe {&mut *logs_ptr};
        let result = core::start(logs);
        result
    });

    // this signals that the process safely(ish--as in it at least unwound that stack) concluded
    match concluded_sender.send(true) {
        Ok(_) => {},
        Err(e) => {
            logs.push(Log {  // not sure how this would actually happen, but at least it's here incase
                message: format!("Failed to send conclusion signal: {}", e), level: LoggingError::Error
            });
        }
    }
    
    match result {
        Ok(Ok(_)) => {},
        Ok(Err(e)) => {
            logs.push(Log {
                message: format!("Fatal Error: {}", e), level: LoggingError::Error
            });
        },
        Err(e) => {
            logs.push(Log {
                message: format!("[Uncaught] Fatal Error: {:?}", e), level: LoggingError::Error
            });
        },
    }
}

