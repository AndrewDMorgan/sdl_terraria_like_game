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

//! Handles the basic application creation and management stuff
//! Actual menues, interactions, gameplay, etc.. will be handled elsewhere

use sdl2::pixels::{PixelFormatEnum};
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use metal::*;

mod shader_handler;
use crate::shader_handler::Shader;

mod game;
use game::Game;

mod textures;
use textures::get_texture_atlas;

// a basic logging function to make reading errors slightly easier
#[derive(serde::Serialize, serde::Deserialize)]
struct Log {
    message: String,
}

// wraps the Log into a vector, but alliased to allow serialization
#[derive(serde::Serialize, serde::Deserialize)]
struct Logs(Vec<Log>);


fn main() -> Result<(), String> {
    // todo! temporary just to handle the game for now, no menues or anything
    let mut _game = Game::new();

    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    
    // Create window
    let window = video
        .window("Name of Game (todo!)", 800, 600)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    
    // --- Create an SDL2 surface and texture ---
    let (_device_width, _device_height) = (video.desktop_display_mode(0)?.w, video.desktop_display_mode(0)?.h);
    let mut window_surface = window.into_canvas().build().unwrap();
    
    let mut event_pump = sdl.event_pump()?;
    
    let mut delta_time: f32 = 0.0;

    // shader stuff (looks so much better when it's wrapped up in its own handler)
    let device = Device::system_default().unwrap();
    let shaders = [
        {
            // wow, having the wrapper handle everything not only cleaned it up, but somehow I haven't gotten a single segfault yet
            let mut shader = Shader::new(&device, "shaders/shader.metal", &[
                size_of::<u32>() as u64, // pitch
                size_of::<u32>() as u64, // width
                size_of::<u32>() as u64, // height
                (size_of::<u32>() * 64 * u16::MAX as usize * 4) as u64, // entity_textures
                (size_of::<u32>() * 64 * u16::MAX as usize * 4) as u64, // tile_textures
                (size_of::<u32>() * 64 * u16::MAX as usize * 4) as u64, // particle_textures
                size_of::<u32>() as u64, // tile_map_width
                size_of::<u32>() as u64, // tile_map_height
                (size_of::<u64>() * 1024 * 1024 * 4) as u64, // tile_map
                (size_of::<f32>() * 3) as u64, // camera_position + rotation
                size_of::<u32>() as u64, // num_entities
                (size_of::<u64>() * 2 * 1024) as u64, // max of 1024 entities on screen at a given time
                size_of::<u32>() as u64, // num_particles
                (size_of::<u64>() * 2 * 2048) as u64, // max of 2048 particles on screen at a given time
                size_of::<u64>() as u64, // num_texts
                ((size_of::<u64>() * 2 + size_of::<u8>() * 32) * 1024) as u64, // max of 1024 text entries on screen at a given time
            ], "ComputeShader").unwrap();
            
            // loading the textures
            const TEXTURE_COUNT: usize = u16::MAX as usize * 4;
            shader.update_buffer(3, get_texture_atlas::<TEXTURE_COUNT>("textures/entities/" , (8, 8))).unwrap(); // entity_textures
            shader.update_buffer(4, get_texture_atlas::<TEXTURE_COUNT>("textures/tiles/"    , (8, 8))).unwrap(); // tile_textures
            shader.update_buffer(5, get_texture_atlas::<TEXTURE_COUNT>("textures/particles/", (8, 8))).unwrap(); // particle_textures

            shader
        }
    ];
    let mut shader_handler = shader_handler::ShaderHandler::new(device, shaders);

    // every time a log is added it should probably be saved
    let mut logs = Logs(Vec::new());
    let _application_start = std::time::Instant::now();
    
    // --- Main loop ---
    let mut frame = 0.0;
    'running: loop {
        // handling events
        let frame_time = std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
        
        let window_size = window_surface.output_size().unwrap();

        // creating a surface to render to (this later will be converted to a texture)
        let mut render_surface = Surface::new(window_size.0, window_size.1, PixelFormatEnum::RGB24).unwrap();
        render_surface.fill_rect(None, sdl2::pixels::Color::RGB(225, 225, 255))?; // Fill with red

        // !====! Do Rendering Here! !====!


        // rendering
        // creating a pixel buffer to pass around to reduce draw calls as the cpu is faster than repeatedly waiting for the gpu to return data
        // the gpu is fast, but data moves between the gpu and cpu slowly
        let (width, height) = (render_surface.width(), render_surface.height());
        let pitch = render_surface.pitch();
        let _buffer_result = render_surface.with_lock_mut::<Result<(), shader_handler::ShaderError>, _>(|pixels| {
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
                }))
            );
            let out_ptr = pixels_buffer.contents() as *mut u8;
            let out_slice = unsafe { std::slice::from_raw_parts(out_ptr, pixels.len()) };
            pixels.copy_from_slice(out_slice);
            Ok(())
        }).unwrap();  // maybe at some point remove the unrwap, but idk, im too lazy rn


        // !====! No Rendering Beyond Here !====!


        // creating a texture to render onto the canvas
        let texture_creator: TextureCreator<WindowContext> = window_surface.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&render_surface)
            .map_err(|e| e.to_string())?;

        // clearing and drawing the texture
        window_surface.clear();
        window_surface.copy(&texture, None, Rect::new(0, 0, window_size.0, window_size.1))?;
        
        // flushing the screen and stuff
        window_surface.present();

        // tracking frame time stuff
        let elapsed = frame_time.elapsed();
        delta_time = elapsed.as_secs_f32();
        frame = frame + delta_time;
        
        //println!("Frame time: {:.3} ms and {:.0} fps", elapsed.as_secs_f32() * 1000.0, 1. / elapsed.as_secs_f64());
    }

    let log_json = serde_json::to_string_pretty(&logs)
        .map_err(|e| e.to_string())?;
    std::fs::write("Logs/logs.json", log_json)
        .map_err(|e| e.to_string())?;

    Ok(())
}

