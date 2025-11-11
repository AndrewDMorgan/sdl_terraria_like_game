use crate::core::{event_handling::event_handler::{self, EventHandler}, timer::Timer};
use crate::logging::logging::{Log, LogType, LoggingError, Logs};
use crate::shaders::shader_handler::{self, ShaderError, Tuple};
use crate::game_manager::game::{Game, GameError};
use crate::textures::textures::get_texture_atlas;
use metal::{MTLSize, NSUInteger};
use std::{rc::Rc};

static MAX_FONT_CHARACTERS: usize = u16::MAX as usize;

pub struct GameStateManager {
    saved_worlds: Vec<String>,
    font_atlas: Rc<Vec<[u32; 256]>>,
    pub game: Option<Game>,
    game_world_name: Option<String>,
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
        let worlds_dir = std::fs::read_dir("world_saves/").map_err(|e| GameError {
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

    pub fn render_ui(&self, pixels: &mut [u8], window_size: (u32, u32), pitch: usize) {
        //
    }

    pub fn handle_events(&mut self, event_handler: &mut EventHandler, event_pump: &mut sdl2::EventPump, timer: &mut Timer, logs: &mut Logs) -> event_handler::Status {
        event_handler.handle_events(event_pump, &mut self.game.as_mut(), &timer)
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
            self.render_ui(pixels, window_size, pitch);
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
