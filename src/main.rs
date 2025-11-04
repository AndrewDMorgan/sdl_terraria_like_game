/*
  To compile:

there's a messy thing going on with homebrew so do:

echo 'export LIBRARY_PATH="/opt/homebrew/lib"' >> ~/.zshrc
echo 'export C_INCLUDE_PATH="/opt/homebrew/include"' >> ~/.zshrc
source ~/.zshrc
export LIBRARY_PATH="/opt/homebrew/lib"
export C_INCLUDE_PATH="/opt/homebrew/include"

this seems to work, idk which parts are necessary, so just run it in every new terminal session before compiling
this should be a problem localized only to my mac, but who knows

*/

use crate::logging::logging::{Log, Logs};

mod shaders;
mod game_manager;
mod textures;
mod logging;
mod core;
mod utill;

fn main() {
    let mut logs = Logs(Vec::new(), false);

    let result = core::start(&mut logs);
    match result {
        Ok(_) => {},
        Err(e) => {
            logs.push(Log { message: format!("Fatal Error: {}", e) });
        }
    }
}

