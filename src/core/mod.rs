use sdl2::pixels::{PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use crate::shaders::*;
use crate::game_manger::game::Game;
use crate::logging::{logging as logger, logging::{Logging, Log, Logs}};
use metal::*;

mod timer;
mod event_handling;
use timer::Timer;

use crate::core::event_handling::*;

/// The starting width of the application window
static WINDOW_START_WIDTH: u32 = 1200;
/// The starting height of the application window
static WINDOW_START_HEIGHT: u32 = 750;


pub fn start() -> Result<(), String> {
    let logging_level = Logging::Everything;
    
    // todo! temporary just to handle the game for now, no menues or anything
    let mut _game = Game::new();

    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    
    // Create window
    let window = video
        .window("Name of Game (todo!)", WINDOW_START_WIDTH, WINDOW_START_HEIGHT)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    
    // --- Create an SDL2 surface and texture ---
    let (_device_width, _device_height) = (video.desktop_display_mode(0)?.w, video.desktop_display_mode(0)?.h);
    let mut window_surface = window.into_canvas().build().map_err(|e| e.to_string())?;
    
    let mut event_pump = sdl.event_pump()?;

    // shader stuff (looks so much better when it's wrapped up in its own handler)
    let device = Device::system_default().ok_or_else(|| String::from("Failed to get system default device"))?;
    let shaders = shader_loader::load_game_shaders(&device)?;
    let mut shader_handler = shader_handler::ShaderHandler::new(device, shaders);

    // every time a log is added it should probably be saved
    let mut logs = Logs(Vec::new(), false);

    // for event stuff
    let mut event_handler = event_handler::EventHandler::new();

    // for handling timing stuff
    let mut timer = Timer::new();
    
    // --- Main loop ---
    'running: loop {
        // handling events
        timer.start_new_frame();
        let status = event_handler.handle_events(event_pump.poll_iter());
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
                    });
                    break 'running;
                }
            }
        }
        
        let elapsed_for_events = timer.elapsed_frame().as_secs_f64();
        
        let window_size = window_surface.output_size()?;

        // creating a surface to render to (this later will be converted to a texture)
        let mut render_surface = Surface::new(window_size.0, window_size.1, PixelFormatEnum::RGB24)?;
        //render_surface.fill_rect(None, sdl2::pixels::Color::RGB(225, 225, 255))?;  // clearing costs a lot.... and the gpu is already running

        let elapsed_for_clearing_surf = timer.elapsed_frame().as_secs_f64();

        // !====! Do Rendering Here! !====!


        // rendering
        // creating a pixel buffer to pass around to reduce draw calls as the cpu is faster than repeatedly waiting for the gpu to return data
        // the gpu is fast, but data moves between the gpu and cpu slowly
        let (width, height) = (render_surface.width(), render_surface.height());
        let pitch = render_surface.pitch();
        let buffer_result = render_surface.with_lock_mut::<Result<(), shader_handler::ShaderError>, _>(|pixels| {
            let grid_size = MTLSize {
                width: width as NSUInteger,
                height: height as NSUInteger,
                depth: 1,
            };
            let threadgroup_size = MTLSize {
                width: 16,
                height: 16,
                depth: 1,
            };
            
            // creating new buffers does incur a cost of about 150-250 microseconds, so avoid doing it alot
            let pixels_buffer =
                shader_handler.device.new_buffer_with_data(
                    pixels.as_mut_ptr() as *mut _ as *mut _,
                    (size_of::<u8>() * pixels.len()) as u64,
                    MTLResourceOptions::StorageModeShared,
            );
            let shader = shader_handler.get_shader(shader_handler::ShaderContext::GameLoop);
            shader.update_buffer(0, pitch as u64)?;
            shader.update_buffer(1, width as u64)?;
            shader.update_buffer(2, height as u64)?;
            shader.execute_with_extra_buffers(
                &[&pixels_buffer],
                grid_size,
                threadgroup_size,
                Some(Box::new(|| {
                    // anything here will run concurrently to the gpu rendering (maybe update tiles or something?)
                    // this is a dyn fn once so it should be able to barrow external variables just fine
                    // this is a slightly odd setup for me; usually I update entities than render, not render than update
                    //    technically speaking, because the gpu requires non-mutating data, even though the updating is
                    //    concurrently happening, it's using the old state of the game, so it kinda does act as render than update
                    //
                }))
            );
            let out_ptr = pixels_buffer.contents() as *mut u8;
            let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, pixels.len()) };
            pixels.copy_from_slice(out_slice);
            Ok(())
        });
        match buffer_result {  // slightly less violently exiting, and at least telling the user why
            Ok(_) => {},
            Err(e) => {
                if matches!(logging_level, Logging::Everything | Logging::ErrorOnly | Logging::WarningOnly) {
                    logs.push(Log {
                        message: format!("[Shader Error] Failed to render frame:\n{:?}", e),
                    });
                    break 'running;
                }
            }
        }

        let elapsed_for_gpu_drawing = timer.elapsed_frame().as_secs_f64();

        // !====! No Rendering Beyond Here !====!


        // creating a texture to render onto the canvas
        let texture_creator: TextureCreator<WindowContext> = window_surface.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&render_surface)
            .map_err(|e| e.to_string())?;

        let elapsed_for_creating_texture = timer.elapsed_frame().as_secs_f64();

        // clearing and drawing the texture
        window_surface.clear();
        window_surface.copy(&texture, None, Rect::new(0, 0, window_size.0, window_size.1))?;

        let elapsed_for_rendering_texture = timer.elapsed_frame().as_secs_f64();

        // flushing the screen and stuff
        window_surface.present();

        let elapsed_for_presenting = timer.elapsed_frame().as_secs_f64();

        // tracking frame time stuff
        timer.update_frame_data();

        // logging slow frames (debug purposes ig)
        if matches!(logging_level, Logging::Everything | Logging::PerformanceOnly) && timer.delta_time > logger::PERFORMANCE_LOG_THRESHOLD {
            let t0 = elapsed_for_events;
            let t1 = elapsed_for_clearing_surf - elapsed_for_events;
            let t2 = elapsed_for_gpu_drawing - elapsed_for_clearing_surf;
            let t3 = elapsed_for_creating_texture - elapsed_for_gpu_drawing;
            let t4 = elapsed_for_rendering_texture - elapsed_for_creating_texture;
            let t5 = elapsed_for_presenting - elapsed_for_rendering_texture;
            let text = format!("Frame timings (ms): Everything: {}, Events: {:.3}, Create Surface: {:.3}, GPU Draw: {:.3}, Create Texture: {:.3}, Render Texture: {:.3}, Present: {:.3}", timer.delta_time * 1000.0, t0 * 1000.0, t1 * 1000.0, t2 * 1000.0, t3 * 1000.0, t4 * 1000.0, t5 * 1000.0);
            logs.push(Log {
                message: format!("[Performance Warning] Frame took too long ( > {:.2}ms  i.e.  < {}fps ).\n{}", logger::PERFORMANCE_LOG_THRESHOLD * 1000.0, 1. / logger::PERFORMANCE_LOG_THRESHOLD, text)
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

