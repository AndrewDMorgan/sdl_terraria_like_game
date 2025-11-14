use crate::core::{event_handling::event_handler::{self, ButtonState, EventHandler}, rendering::ui::{UiElement, UiError}, timer::Timer};
use crate::game_manager::entities::player::font_rendering::render_font_unifont_colored;
use crate::logging::logging::{Log, LogType, LoggingError, Logs};
use crate::shaders::shader_handler::{self, ShaderError, Tuple};
use crate::game_manager::game::{Game, GameError};
use crate::textures::textures::get_texture_atlas;
use metal::{MTLSize, NSUInteger};
use std::{fs::ReadDir, rc::Rc};

static MAX_FONT_CHARACTERS: usize = u16::MAX as usize;

struct CreatorUi {
    ui_element: UiElement<Option<String>>,
}

impl CreatorUi {
    pub fn new(window_size: (u32, u32), font_atlas: Rc<Vec<[u32; 256]>>) -> Self {
        Self {
            ui_element: UiElement::new(
                String::from("Creator"),
                (125, 125),
                (window_size.0 as usize - 250, 500),
                Box::new(move |pixels, (window_size, pitch), data, (pos, _size)| {
                    // rendering the boxes outline
                    let size = (window_size.0 as usize - 250, 500);
                    for x in pos.0+1..pos.0 + size.0 {
                        pixels[x * 3 + pos.1 * pitch    ] = 225;
                        pixels[x * 3 + pos.1 * pitch + 1] = 225;
                        pixels[x * 3 + pos.1 * pitch + 2] = 225;
                        for y in pos.1+1..pos.1 + size.1 {
                            pixels[pos.0 * 3 + y * pitch    ] = 225;
                            pixels[pos.0 * 3 + y * pitch + 1] = 225;
                            pixels[pos.0 * 3 + y * pitch + 2] = 225;
                            
                            pixels[(pos.0 + size.0) * 3 + y * pitch    ] = 225;
                            pixels[(pos.0 + size.0) * 3 + y * pitch + 1] = 225;
                            pixels[(pos.0 + size.0) * 3 + y * pitch + 2] = 225;

                            pixels[x * 3 + y * pitch    ] = 75;
                            pixels[x * 3 + y * pitch + 1] = 75;
                            pixels[x * 3 + y * pitch + 2] = 75;
                        }
                        pixels[x * 3 + (pos.1 + size.1) * pitch    ] = 225;
                        pixels[x * 3 + (pos.1 + size.1) * pitch + 1] = 225;
                        pixels[x * 3 + (pos.1 + size.1) * pitch + 2] = 225;
                    }

                    render_font_unifont_colored::<16, 256, 8> (
                        &*font_atlas,
                        pixels,
                        (pos.0 + 25, pos.1 + 25),
                        window_size,
                        pitch,
                        &data.as_ref().unwrap_or(&String::from("New World")),
                        255u32 | (255u32 << 8) | (255u32 << 16)
                    );

                    render_font_unifont_colored::<16, 256, 8> (
                        &*font_atlas,
                        pixels,
                        (pos.0 + size.0 - 115, pos.1 + size.1 - 25),
                        window_size,
                        pitch,
                        "Create World",
                        255u32 | (255u32 << 8) | (255u32 << 16)
                    );

                    Ok(())
                })
            ),
        }
    }
}

pub struct GameStateManager {
    saved_worlds: Vec<String>,
    font_atlas: Rc<Vec<[u32; 256]>>,
    pub game: Option<Game>,
    game_world_name: Option<String>,
    creator_popup: Option<CreatorUi>,
}

impl GameStateManager {
    pub fn new(logs: &mut Logs) -> Result<Self, GameError> {
        let font_atlas = Rc::new({
            let mut total_textures_loaded = 0;
            let atlas = get_texture_atlas::<MAX_FONT_CHARACTERS, 256>(
                "textures/fonts/user_default/", (16, 16), vec![[0u32; 256]; MAX_FONT_CHARACTERS], &mut total_textures_loaded
            ).map_err(|e| GameError {
                message: format!("[Ui-Manager Game Error] Failed to load textures: {:?}", e),
                severity: crate::game_manager::game::Severity::Fatal
            })?;
            logs.push(Log {
                message: format!("Loaded {} font characters for cpu-side rendering.", total_textures_loaded - 1),
                level: crate::logging::logging::LoggingError::Info
            }, 20, LogType::Information);
            atlas
        });
        // going through the game save file path to look for saved worlds
        let worlds_dir: Result<ReadDir, std::io::Error> = match std::fs::read_dir("world_saves/") {
            Ok(dir) => Ok(dir),
            Err(_e) => {
                // creating the directory if it doesn't exist, then reading it again
                match std::fs::create_dir("world_saves/") {
                    Err(e) => Err(e),
                    Ok(_) => std::fs::read_dir("world_saves/")
                }
            }
        };
        let worlds_dir = worlds_dir.map_err(|e| GameError {
            message: format!("[Ui-Manager Game Error] failed to read world saves: {:?}", e),
            severity: crate::game_manager::game::Severity::Fatal,
        })?;
        let mut saved_worlds = vec![];
        for world in worlds_dir {
            if let Ok(entry) = world {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        saved_worlds.push(entry.file_name().to_string_lossy().into_owned());
                    }
                }
            }
        }
        Ok(Self {
            saved_worlds,
            font_atlas,
            game: None,
            game_world_name: None,
            creator_popup: None,
        })
    }

    pub fn start_game(&mut self, world_name: Option<String>, logs: &mut Logs, game_version: &str) -> Result<(), GameError> {
        if let Some(world_name) = world_name {
            let game = Game::from_save(logs, &format!("world_saves/{}", world_name), game_version, self.font_atlas.clone())?;
            self.game_world_name = Some(world_name);
            self.game = Some(game);
        } else {
            self.game = Some(Game::new(logs, self.font_atlas.clone())?);
            self.game_world_name = None;
        }
        Ok(())
    }

    pub fn render_ui(&mut self, pixels: &mut [u8], window_size: (u32, u32), pitch: usize) -> Result<(), UiError> {
        render_font_unifont_colored::<16, 256, 8> (
            &*self.font_atlas,
            pixels,
            (25, 25),
            window_size,
            pitch,
            "World Saves:",
            255u32 | (255u32 << 8) | (255u32 << 16)
        );
        render_font_unifont_colored::<16, 256, 8> (
            &*self.font_atlas,
            pixels,
            (window_size.0 as usize - 150, 25),
            window_size,
            pitch,
            "New World",
            255u32 | (255u32 << 8) | (255u32 << 16)
        );
        //rendering the currently available worlds
        for (index, save_name) in self.saved_worlds.iter().enumerate() {
            render_font_unifont_colored::<16, 256, 8> (
                &*self.font_atlas,
                pixels,
                (50, 55 + index * 20),
                window_size,
                pitch,
                save_name,
                255u32 | (255u32 << 8) | (255u32 << 16)
            );
        }
        
        if let Some(creator) = self.creator_popup.as_ref() {
            creator.ui_element.render(pixels, window_size, pitch, &mut self.game_world_name)?;
        }
        Ok(())
    }

    fn handle_menue_events(&mut self, event_handler: &mut EventHandler, _timer: &mut Timer, logs: &mut Logs, window_size: (u32, u32), game_version: &str) -> Result<(), GameError> {
        if matches!(event_handler.mouse.left, ButtonState::Released) {
            // checking what was clicked
            let mouse_pos = event_handler.mouse.position;
            if self.creator_popup.is_none() && mouse_pos.0 > 50 && mouse_pos.0 < window_size.0 - 150 && mouse_pos.1 > 55 {
                // clicked a world (unless the index is beyond the total count of worlds)
                let index = (mouse_pos.1 - 55) as usize / 20;
                if index < self.saved_worlds.len() {
                    // opening the world
                    self.start_game(Some(self.saved_worlds[index].clone()), logs, game_version)?;
                }
            } else if self.creator_popup.is_none() && mouse_pos.0 >= window_size.0 - 150 && mouse_pos.1 < 45 && mouse_pos.1 > 20 {
                self.creator_popup = Some(CreatorUi::new(window_size, self.font_atlas.clone()));
            } else if self.creator_popup.is_some() && mouse_pos.0 >= window_size.0 - 125 - 115 && mouse_pos.1 >= 500 - 25 + 125 && mouse_pos.0 < window_size.0 - 125 && mouse_pos.1 < 500 + 125 {
                self.creator_popup = None;
                let name = self.game_world_name.take();
                if let Some(name) = name.as_ref() {
                    // creating the directory so it doesn't crash on save
                    std::fs::create_dir_all(&format!("world_saves/{}/game_version_{}/world_save/entities", name, game_version)).map_err(|e| GameError {
                        message: format!("[World Creation Error] Failed to create world directory: {:?}", e),
                        severity: crate::game_manager::game::Severity::Fatal
                    })?;
                    std::fs::create_dir_all(&format!("world_saves/{}/game_version_{}/player", name, game_version)).map_err(|e| GameError {
                        message: format!("[World Creation Error] Failed to create world directory: {:?}", e),
                        severity: crate::game_manager::game::Severity::Fatal
                    })?;
                }
                self.start_game(None, logs, game_version)?;
                self.game_world_name = name;
                // saving the world (for one, this makes sure everything is correctly setup before the user actually gets invested into the world)
                if let Some(game) = self.game.as_mut() {
                    if let Some(name) = self.game_world_name.as_ref() {
                        game.save(&format!("world_saves/{}", name), game_version, logs).map_err(|e| GameError {
                            message: format!("[World Creation Error] Failed to save the new world: {:?}", e),
                            severity: crate::game_manager::game::Severity::Fatal
                        })?;
                    }
                }
            }
        }
        if self.creator_popup.is_some() {
            // typing ig
            for char in "abcdefghijklmnopqrstuvwxyz- 1234567890".chars() {
                if let Some(code) = sdl2::keyboard::Keycode::from_name(&char.to_string()) {
                    if event_handler.keys_released.contains(&code) {
                        if self.game_world_name.is_none() { self.game_world_name.replace(String::new()); }
                        let char = if event_handler.mods_released.contains(&sdl2::keyboard::Mod::LSHIFTMOD) || event_handler.mods_released.contains(&sdl2::keyboard::Mod::RSHIFTMOD) ||
                                            event_handler.mods_pressed .contains(&sdl2::keyboard::Mod::LSHIFTMOD) || event_handler.mods_pressed .contains(&sdl2::keyboard::Mod::RSHIFTMOD) {
                            match char {
                                '-' => '_',
                                '1' => '!',
                                '2' => '@',
                                '3' => '#',
                                '4' => '$',
                                '5' => '%',
                                '6' => '^',
                                '7' => '&',
                                '8' => '*',
                                '9' => '(',
                                '0' => ')',
                                _ => char.to_ascii_uppercase(),
                            }
                        } else { char };
                        self.game_world_name.as_mut().ok_or_else(|| GameError {
                            message: String::from("Failed to write to the new world's name."),
                            severity: crate::game_manager::game::Severity::Fatal,
                        })?.push(char);
                    }
                }
            }
            if event_handler.keys_released.contains(&sdl2::keyboard::Keycode::BACKSPACE) || event_handler.keys_released.contains(&sdl2::keyboard::Keycode::DELETE) {
                if self.game_world_name.is_none() { self.game_world_name.replace(String::new()); }
                self.game_world_name.as_mut().ok_or_else(|| GameError {
                    message: String::from("Failed to write to the new world's name."),
                    severity: crate::game_manager::game::Severity::Fatal,
                })?.pop();
            }
        }
        Ok(())
    }

    pub fn handle_events(
        &mut self,
        event_handler: &mut EventHandler,
        event_pump: &mut sdl2::EventPump,
        timer: &mut Timer,
        logs: &mut Logs,
        window_size: (u32, u32),
        game_version: &str
    ) -> Result<event_handler::Status, GameError> {
        let status = event_handler.handle_events(event_pump, &mut self.game.as_mut(), &timer);
        if self.game.is_none() {
            // updating any ui (such as clicks and stuff)
            self.handle_menue_events(event_handler, timer, logs, window_size, game_version)?;
        }
        Ok(status)
    }

    pub fn update_entities(&mut self, entities: &mut Vec<Vec<(u32, u16, i16, i16, u16, u32)>>, window_size: (u32, u32)) {
        if let Some(game) = self.game.as_mut() {
            entities.push(game.player.get_model());
            entities.push(game.entity_manager.get_render(&game.player.camera, window_size));
        }
    }

    pub fn update_text_buffer(&mut self, text_buffer: &mut Vec<Tuple<u128, [u8; 32]>>) {
        if let Some(game) = self.game.as_mut() {
            text_buffer.push({
                let input_text = format!(
                    "({},{})",
                    (game.player.entity.position.0 / 8.0) as usize,
                    match game.get_tilemap_manager().get_current_map(crate::game_manager::world::tile_map::Dimension::Overworld) {
                        Some(tile_map) => (tile_map.get_map_height() - 1) as usize,
                        None => 0,
                    } - (game.player.entity.position.1 / 8.0) as usize
                );
                Tuple {
                    first:
                        100u128 << 112 |  // x offset
                        90u128  << 96  |  // y offset
                        ((u16::MAX as u128) << 48) |  // color
                        16u128  << 8   |  // font size
                        input_text.len() as u128,  // buffer size
                    second: {
                        let mut text = [0u8; 32];
                        for (i, char) in input_text.chars().enumerate() {
                            text[i] = char as u8;
                        }
                        text
                    },
                }
            })
        }
    }

    pub fn execute_shader(
        &mut self,
        event_handler: &mut EventHandler,
        window_size: (u32, u32),
        pixels: &mut [u8],
        shader: &mut shader_handler::Shader,
        logs: &mut Logs,
        updating_error: &mut Result<(), GameError>,
        timer: &mut Timer,
        pitch: usize,
        buffer_upload_end: &mut f64,
        shader_render_pass_time_start: &mut f64,
        shader_render_pass_time_end: &mut f64,
        ui_rendering_time_start: &mut f64,
        start_of_event_handling: &mut f64,
        elapsed_for_event_handling: &mut f64,
    ) -> Result<(), ShaderError> {
        if let Some(game) = self.game.as_mut() {
            let grid_size = MTLSize {
                width: window_size.0 as NSUInteger,
                height: window_size.1 as NSUInteger,
                depth: 1,
            };
            let threadgroup_size = MTLSize {
                width: 16,
                height: 16,
                depth: 1,
            };

            // getting the tilemap slice to render
            let camera = &game.player.camera.clone();  // the struct is only a couple 32 bit floats or whatever, so not too expensive to clone
            match game.get_tilemap_manager().get_current_map(crate::game_manager::world::tile_map::Dimension::Overworld) {
                Some(tile_map) => {
                    let (map, offset_transform, visible_size) = tile_map.get_render_slice(
                        camera,
                        window_size,
                    );
                    shader.update_buffer_slice(8, &map)?;
                    shader.update_buffer(6, visible_size.0)?;
                    shader.update_buffer(7, visible_size.1)?;
                    let transform = shader_handler::Float4::new(offset_transform.x, offset_transform.y, offset_transform.zoom, 0.0);
                    shader.update_buffer(9, transform)?;
                },
                _ => {
                    // if there's none, do this?
                    shader.update_buffer_slice::<&[u32]>(8, &[])?;
                    logs.push(Log {
                        message: format!("[Render Warning] No tile map found for rendering in current dimension."),
                        level: LoggingError::Warning,
                    }, 15, LogType::Warning);
                }
            }

            shader.update_buffer_slice(18, pixels)?;

            *buffer_upload_end = timer.elapsed_frame().as_secs_f64();
            *shader_render_pass_time_start = timer.elapsed_frame().as_secs_f64();

            *updating_error = shader.execute(
                grid_size,
                threadgroup_size,
                Some(Box::new(|| {
                    *start_of_event_handling = timer.elapsed_frame().as_secs_f64();
                    // anything here will run concurrently to the gpu rendering (maybe update tiles or something?)
                    // this is a dyn fn once so it should be able to barrow external variables just fine
                    // this is a slightly odd setup for me; usually I update entities than render, not render than update
                    //    technically speaking, because the gpu requires non-mutating data, even though the updating is
                    //    concurrently happening, it's using the old state of the game, so it kinda does act as render than update
                    game.update_key_events(&timer, &event_handler, window_size, logs)?;

                    *elapsed_for_event_handling = timer.elapsed_frame().as_secs_f64();
                    Ok(())
                }))
            );
            let out_ptr = shader.get_buffer_contents(18);
            let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, pixels.len()) };
            pixels.copy_from_slice(out_slice);

            *shader_render_pass_time_end = timer.elapsed_frame().as_secs_f64();
            *ui_rendering_time_start = timer.elapsed_frame().as_secs_f64();

            // rendering ui stuff
            game.render_ui(pixels, window_size, pitch).map_err(|e| {
                ShaderError::new(
                    format!("[Ui Error] Error while rendering ui: {:?}", e)
                )
            })?;
        } else {
            // rendering the main menue ui
            self.render_ui(pixels, window_size, pitch).map_err(|e| ShaderError {
                details: format!("{:?}", e)
            })?;
        } Ok(())
    }

    pub fn close_game_session(&mut self, game_version: &str, logs: &mut Logs) -> Result<(), String> {
        if let Some(game) = self.game.as_mut() {
            if let Some(name) = self.game_world_name.as_ref() {
                game.save(&format!("world_saves/{}", name), game_version, logs).map_err(|e| format!("{:?}", e))?;
            }
        }
        Ok(())
    }
}
