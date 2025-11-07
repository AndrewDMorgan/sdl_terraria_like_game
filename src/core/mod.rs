use sdl2::render::{TextureAccess, TextureCreator};
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use metal::*;

use crate::logging::logging::LoggingError;
use crate::logging::{logging as logger, logging::{Logging, Log, Logs}};
use crate::game_manager::game::{Game, GameError, Severity};
use crate::shaders::shader_handler::{ShaderError, Tuple};

use crate::core::event_handling::*;
pub(crate) mod event_handling;
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

pub fn start(logs: &mut Logs) -> Result<(), String> {
    // this could, in theory, be configurable, but for now, debugging is necessary so this is fine (and there's no ui and therefore no settings...)
    let logging_level = Logging::Everything;
    
    // todo! temporary just to handle the game for now, no menues or anything
    let mut game = Game::new(logs)?;

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
        let status = event_handler.handle_events(&mut event_pump, &mut Some(&mut game), &timer);
        match status {
            event_handler::Status::Continue => {},
            event_handler::Status::Quit => break 'running,
            event_handler::Status::Error(ref msg, severity) => {
                if match (&logging_level, severity) {
                    (Logging::Everything, _) => true,
                    (Logging::WarningOnly, 0..15 | u8::MAX) => true,
                    (Logging::ErrorOnly, 15..) => true,
                    (Logging::PerformanceOnly, 75..) => true,
                    (Logging::Nothing, u8::MAX) => false,
                    _ => false,
                } {
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
                    });
                    if severity == u8::MAX {  break 'running;  }
                }
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
            });
            break 'running;
        }

        // !====! Do Rendering Here! !====!


        // rendering
        // creating a pixel buffer to pass around to reduce draw calls as the cpu is faster than repeatedly waiting for the gpu to return data
        // the gpu is fast, but data moves between the gpu and cpu slowly
        let mut elapsed_for_event_handling = 0.0;
        let mut start_of_event_handling = 0.0;
        let mut updating_error: Result<(), GameError> = Ok(());
        let buffer_result: Result<(), ShaderError> = surface_texture.with_lock(None, |pixels, pitch| {
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
            
            let shader = shader_handler.get_shader(shader_handler::ShaderContext::GameLoop);
            shader.update_buffer(0, pitch as u64)?;
            shader.update_buffer(1, window_size.0 as u64)?;
            shader.update_buffer(2, window_size.1 as u64)?;

            let mut entities: Vec<Vec<(u32, u16, i16, i16, u16, u32)>> = vec![];
            entities.push(game.player.get_model());
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
            shader.update_buffer(10, entities.len() as u32)?;
            shader.update_buffer_slice(11, &entities)?;
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
            let text_buffer = vec![
                Tuple {
                    first:
                        100u128 << 112 |  // x offset
                        90u128  << 96  |  // y offset
                        (((190u32 << 16) as u128) << 48) |  // color
                        8u128  << 8   |  // font size
                        12u128,  // buffer size
                    second: {
                        let mut text = [0u8; 32];
                        text[0 ] = 'H' as u8;
                        text[1 ] = 'e' as u8;
                        text[2 ] = 'l' as u8;
                        text[3 ] = 'l' as u8;
                        text[4 ] = 'o' as u8;
                        text[5 ] = ' ' as u8;
                        text[6 ] = 'W' as u8;
                        text[7 ] = 'o' as u8;
                        text[8 ] = 'r' as u8;
                        text[9 ] = 'l' as u8;
                        text[10] = 'd' as u8;
                        text[11] = '!' as u8;
                        text
                    },
                }
            ];
            shader.update_buffer(14, text_buffer.len() as u32)?;
            shader.update_buffer_slice(15, &text_buffer)?;

            // getting the tilemap slice to render
            let camera = &game.player.camera;
            match game.get_tilemap_manager_ref().get_current_map_ref(crate::game_manager::world::tile_map::Dimension::Overworld) {
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
                    if matches!(logging_level, Logging::Everything | Logging::WarningOnly) {
                        logs.push(Log {
                            message: format!("[Render Warning] No tile map found for rendering in current dimension."),
                            level: LoggingError::Warning,
                        });
                    }
                },
            }

            shader.update_buffer_slice(18, pixels)?;
            updating_error = shader.execute(
                grid_size,
                threadgroup_size,
                Some(Box::new(|| {
                    start_of_event_handling = timer.elapsed_frame().as_secs_f64();
                    // anything here will run concurrently to the gpu rendering (maybe update tiles or something?)
                    // this is a dyn fn once so it should be able to barrow external variables just fine
                    // this is a slightly odd setup for me; usually I update entities than render, not render than update
                    //    technically speaking, because the gpu requires non-mutating data, even though the updating is
                    //    concurrently happening, it's using the old state of the game, so it kinda does act as render than update
                    game.update_key_events(&timer, &event_handler, window_size)?;

                    elapsed_for_event_handling = timer.elapsed_frame().as_secs_f64();
                    Ok(())
                }))
            );
            let out_ptr = shader.get_buffer_contents(18);
            let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, pixels.len()) };
            pixels.copy_from_slice(out_slice);

            // rendering ui stuff
            println!("pitch {} width {}", pitch, window_size.0);
            game.render_ui(pixels, window_size, pitch).map_err(|e| {
                ShaderError::new(
                    format!("[Ui Error] Error while rendering ui: {:?}", e)
                )
            })?;
            
            Ok(())
        })?;
        match buffer_result {  // slightly less violently exiting, and at least telling the user why
            Ok(_) => {},
            Err(e) => {
                if matches!(logging_level, Logging::Everything | Logging::ErrorOnly | Logging::WarningOnly) {
                    logs.push(Log {
                        message: format!("[Shader Error] Failed to render frame:\n{:?}", e),
                        level: LoggingError::Error,
                    });
                    break 'running;
                }
            }
        }
        match updating_error {
            Ok(_) => {},
            Err(e) => {
                if matches!(logging_level, Logging::Everything | Logging::ErrorOnly | Logging::WarningOnly) {
                    logs.push(Log {
                        message: format!("[Game Update Error] Failed to update game state during frame render:\n{:?}", e),
                        level: LoggingError::Error,
                    });
                    if e.severity == Severity::Fatal { break 'running; }
                }
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
        if matches!(logging_level, Logging::Everything | Logging::PerformanceOnly) && timer.delta_time > logger::PERFORMANCE_LOG_THRESHOLD {
            let t0 = elapsed_for_events;
            let t2 = elapsed_for_gpu_drawing - elapsed_for_events;
            let t6 = elapsed_for_event_handling - start_of_event_handling;
            let t4 = elapsed_for_rendering_texture - elapsed_for_events;
            let t5 = elapsed_for_presenting - elapsed_for_rendering_texture;
            let text = format!("Frame timings (ms): Everything: {:.3}, Events: {:.3}, [ GPU Draw: {:.3} ; Event Handling: {:.3} ], Render Texture: {:.3}, Present: {:.3}", timer.delta_time * 1000.0, t0 * 1000.0, t2 * 1000.0, t6 * 1000.0, t4 * 1000.0, t5 * 1000.0);
            logs.push(Log {
                message: format!("[Performance Warning] Frame took too long ( > {:.2}ms  i.e.  < {}fps ).\n{}", logger::PERFORMANCE_LOG_THRESHOLD * 1000.0, 1. / logger::PERFORMANCE_LOG_THRESHOLD, text),
                level: LoggingError::Warning,
            });
        }

        // only doing it here in case multiple logs are added in a frame
        if logs.was_updated() {
            logs.save()?;
        }

        //println!("Frame time: {:.3} ms and {:.0} fps", elapsed.as_secs_f32() * 1000.0, 1. / elapsed.as_secs_f64());
    }

    logs.save()?;

    Ok(())
}

