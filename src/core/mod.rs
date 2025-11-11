use sdl2::render::{TextureAccess, TextureCreator};
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use metal::*;

use crate::core::rendering::ui_state::menues::GameStateManager;
use crate::logging::logging::LoggingError;
use crate::logging::{logging as logger, logging::{Log, Logs}};
use crate::game_manager::game::{GameError, Severity};
use crate::shaders::shader_handler::{ShaderError};

use crate::core::event_handling::*;
pub(crate) mod event_handling;
use crate::shaders::shader_loader::MAX_ENTITIES;
use crate::shaders::*;
pub(crate) mod timer;
use timer::Timer;

pub(crate) mod rendering;

/// The starting width of the application window
static WINDOW_START_WIDTH: u32 = 1200;
/// The starting height of the application window
static WINDOW_START_HEIGHT: u32 = 750;

/// The minimum size of the window (mostly so ui doesn't get completely messed up)
static MINIMUM_WINDOW_WIDTH: u32 = 1200;
/// The minimum size of the window (mostly so ui doesn't get completely messed up)
static MINIMUM_WINDOW_HEIGHT: u32 = 750;

static GAME_VERSION: &'static str = "0.0.1-alpha";

pub fn start(logs: &mut Logs) -> Result<(), String> {
    // todo! temporary just to handle the game for now, no menues or anything (umm..... it does have some ui now... but we'll go with that)
    //     *temporary apparently means permanent? Either way, it's here to stay
    let mut game_manager = GameStateManager::new(logs)?;  // handles everything, making it easier to have multiple menue states and
    // current save: Some("testing_world".to_string())
    game_manager.start_game(None, logs, GAME_VERSION)?;
    
    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    
    // Create window
    let mut window = video
        .window("Name of Game (todo!)", WINDOW_START_WIDTH, WINDOW_START_HEIGHT)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    window.set_minimum_size(MINIMUM_WINDOW_WIDTH, MINIMUM_WINDOW_HEIGHT)
        .map_err(|e| e.to_string())?;
    
    // --- Create an SDL2 surface and texture ---
    let (device_width, device_height) = (video.desktop_display_mode(0)?.w, video.desktop_display_mode(0)?.h);
    let mut window_surface = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    
    // creating the texture that all runtime drawing will be done to
    // this texture will than be uploaded onto the window_surface
    let texture_creator: TextureCreator<WindowContext> = window_surface.texture_creator();
    let mut surface_texture = texture_creator
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming,   WINDOW_START_WIDTH, WINDOW_START_HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut surface_texture_size = (WINDOW_START_WIDTH, WINDOW_START_HEIGHT);

    let mut event_pump = sdl.event_pump()?;

    // shader stuff (looks so much better when it's wrapped up in its own handler)
    let device = Device::system_default()
        .ok_or_else(|| String::from("Failed to get system default device"))?;
    let shaders = shader_loader::load_game_shaders(&device, (device_width as u32, device_height as u32), logs)?;
    let mut shader_handler = shader_handler::ShaderHandler::new(device, [shaders]);
    
    // for event stuff
    let mut event_handler = event_handler::EventHandler::new();

    // for handling timing stuff
    let mut timer = Timer::new();
    
    // --- Main loop ---
    'running: loop {
        // handling events
        timer.start_new_frame();
        let status = game_manager.handle_events(&mut event_handler, &mut event_pump, &mut timer, logs);
        match status {
            event_handler::Status::Continue => {},
            event_handler::Status::Quit => break 'running,
            event_handler::Status::Error(ref msg, severity) => {
                logs.push(Log {
                    message: format!("[Event Handling Error; {}] {}", match severity {
                        0..15 => "Warning",
                        15..75 => "Error",
                        75..u8::MAX => "Serious",
                        u8::MAX => "Fatal",
                    }, msg),
                    level: match severity {
                        0..15 => LoggingError::Warning,
                        _ => LoggingError::Error,
                    },
                }, 12, match severity {
                        0..15 => logger::LogType::Warning,
                        _ => logger::LogType::Error,
                    });
                if severity == u8::MAX {  break 'running;  }
            }
        }
        
        let elapsed_for_events = timer.elapsed_frame().as_secs_f64();
        
        // checking the surface texture's size
        let window_size = window_surface.output_size()?;
        if surface_texture_size != window_size {
            surface_texture = texture_creator
                .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, window_size.0, window_size.1)
                .map_err(|e| e.to_string())?;
            surface_texture_size = window_size;
        }

        // should hopefully prevent segfaults from buffer overflows? Honestly, not sure how this would happen though
        if window_size.0 > device_width as u32 || window_size.1 > device_height as u32 {
            logs.push(Log {
                message: format!("[Buffer Overflow Error; Fatal] Window size ({}, {}) exceeds the expected buffer size ({}, {}). Unable to continue as it will overflow the buffer.", window_size.0, window_size.1, device_width, device_height),
                level: LoggingError::Error,
            }, 13, logger::LogType::Error);
            break 'running;
        }

        // !====! Do Rendering Here! !====!


        // rendering
        // creating a pixel buffer to pass around to reduce draw calls as the cpu is faster than repeatedly waiting for the gpu to return data
        // the gpu is fast, but data moves between the gpu and cpu slowly
        let mut elapsed_for_event_handling = 0.0;
        let mut start_of_event_handling = 0.0;
        let mut shader_render_pass_time_start = 0.0;
        let mut shader_render_pass_time_end = 0.0;
        let mut ui_rendering_time_start = 0.0;
        let mut ui_rendering_time_end = 0.0;
        let mut buffer_upload_start = 0.0;
        let mut buffer_upload_end = 0.0;
        let gpu_start = timer.elapsed_frame().as_secs_f64();
        let mut updating_error: Result<(), GameError> = Ok(());
        let buffer_result: Result<(), ShaderError> = surface_texture.with_lock(None, |pixels, pitch| {
            buffer_upload_start = timer.elapsed_frame().as_secs_f64();

            let mut shader = shader_handler.get_shader(shader_handler::ShaderContext::GameLoop);
            shader.update_buffer(0, pitch as u64)?;
            shader.update_buffer(1, window_size.0 as u64)?;
            shader.update_buffer(2, window_size.1 as u64)?;

            let mut entities: Vec<Vec<(u32, u16, i16, i16, u16, u32)>> = vec![];
            game_manager.update_entities(&mut entities, window_size);
            if entities.len() >= MAX_ENTITIES {
                logs.push(Log {
                    message: format!("[Memory Warning] Entities surpassed maximum GPU buffer size; length of {}", entities.len()),
                    level: LoggingError::Warning
                }, 14, logger::LogType::Memory);
            }

            let entities = entities
                .concat()
                .iter()
                .map(|(texture_id, rot, offset_x, offset_y, _padding, depth)|
            {
                ((*texture_id as u128) << 96) |
                ((*rot as u128) << 80) |
                ((offset_x.cast_unsigned() as u128) << 64) |
                ((offset_y.cast_unsigned() as u128) << 48) |
                (*depth as u128)
            }).collect::<Vec<u128>>();
            shader.update_buffer(10, entities.len().min(MAX_ENTITIES) as u32)?;
            // making sure the slice doesn't overflow or anything
            shader.update_buffer_slice(11, &entities[0..entities.len().min(MAX_ENTITIES)])?;
            /*
            constant uint&   num_texts         [[ buffer(14) ]],  // number of text entries
            constant Text*   text_buffer       [[ buffer(15) ]],  // text data
            text buffer for rendering text is a 128 bit value and a buffer of 32 u8:
            //        16 -> x offset (screen space; top left of text, uint)
            //        16 -> y offset (screen space; top left of text, uint)
            //        16 -> rotation
            //        32 -> color (r, g, b each being 8, 8, 8, 8 for alpha)
            //        32 -> applicable data (not sure what would go here yet, but it's reserved anyways, so use if needed)
            //        8 bits for font size
            //        8 bits for the length of the character buffer
            */
            let mut text_buffer = vec![];
            game_manager.update_text_buffer(&mut text_buffer);
            shader.update_buffer(14, text_buffer.len() as u32)?;
            shader.update_buffer_slice(15, &text_buffer)?;

            game_manager.execute_shader(
                &mut event_handler,
                window_size,
                pixels,
                &mut shader,
                logs,
                &mut updating_error,
                &mut timer,
                pitch,
                &mut buffer_upload_end,
                &mut shader_render_pass_time_start,
                &mut shader_render_pass_time_end,
                &mut ui_rendering_time_start,
                &mut start_of_event_handling,
                &mut elapsed_for_event_handling,
            )?;

            ui_rendering_time_end = timer.elapsed_frame().as_secs_f64();
            
            Ok(())
        })?;
        match buffer_result {  // slightly less violently exiting, and at least telling the user why
            Ok(_) => {},
            Err(e) => {
                logs.push(Log {
                    message: format!("[Shader Error] Failed to render frame:\n{:?}", e),
                    level: LoggingError::Error,
                }, 16, logger::LogType::Error);
                break 'running;
            }
        }
        match updating_error {
            Ok(_) => {},
            Err(e) => {
                logs.push(Log {
                    message: format!("[Game Update Error] Failed to update game state during frame render:\n{:?}", e),
                    level: LoggingError::Error,
                }, 17, logger::LogType::Error);
                if e.severity == Severity::Fatal { break 'running; }
            }
        }
        
        let elapsed_for_gpu_drawing = timer.elapsed_frame().as_secs_f64();

        // !====! No Rendering Beyond Here !====!

        // clearing and drawing the texture
        window_surface.clear();
        window_surface.copy(&surface_texture, None, Rect::new(0, 0, window_size.0, window_size.1))?;

        let elapsed_for_rendering_texture = timer.elapsed_frame().as_secs_f64();

        // flushing the screen and stuff
        window_surface.present();

        let elapsed_for_presenting = timer.elapsed_frame().as_secs_f64();

        // tracking frame time stuff
        timer.update_frame_data();

        // logging slow frames (debug purposes ig)
        if timer.delta_time > logger::PERFORMANCE_LOG_THRESHOLD {
            let t0 = elapsed_for_events;
            let t2 = elapsed_for_gpu_drawing - gpu_start;
            let t6 = elapsed_for_event_handling - start_of_event_handling;
            let t4 = elapsed_for_rendering_texture - elapsed_for_events;
            let t5 = elapsed_for_presenting - elapsed_for_rendering_texture;

            let shader_render_time = shader_render_pass_time_end - shader_render_pass_time_start;
            let ui_render_time = ui_rendering_time_end - ui_rendering_time_start;
            let buffer_upload_time = buffer_upload_end - buffer_upload_start;

            let text = format!("Frame timings (ms): Everything: {:.3}, Events: {:.3}, [ GPU Draw: {:.3} ; Buffer Upload: {:.3}, Shader Render: {:.3}, Ui Rendering: {:.3} Event Handling: {:.3} ], Render Texture: {:.3}, Present: {:.3}",
                timer.delta_time * 1000.0,  // everything
                t0 * 1000.0,  // events
                t2 * 1000.0,  // gpu draw
                buffer_upload_time * 1000.0,  // buffer upload
                shader_render_time * 1000.0,  // shader render
                ui_render_time * 1000.0,  // ui render
                t6 * 1000.0,  // event handling (yes, it reads 0, but it does actually do things and therefore should be here)
                t4 * 1000.0,  // render texture
                t5 * 1000.0,  // present
            );
            logs.push(Log {
                message: format!("[Performance Warning] Frame took too long ( > {:.2}ms  i.e.  < {}fps ).\n{}\n * entity count: {}", logger::PERFORMANCE_LOG_THRESHOLD * 1000.0, 1. / logger::PERFORMANCE_LOG_THRESHOLD, text, {
                    match &game_manager.game {
                        Some(game) => game.entity_manager.get_entity_count(),
                        None => 0,
                    }
                }),
                level: LoggingError::Warning,
            }, 18, logger::LogType::Performance);
        }

        // only doing it here in case multiple logs are added in a frame
        if logs.was_updated() {
            logs.save()?;
        }
    }

    logs.save()?;

    game_manager.close_game_session(GAME_VERSION, logs)?;

    Ok(())
}

