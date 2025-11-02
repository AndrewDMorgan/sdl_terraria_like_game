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

mod shaders;
mod game_manager;
mod textures;
mod logging;
mod core;

fn main() -> Result<(), String> {
    core::start()
}

