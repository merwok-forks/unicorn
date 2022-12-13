mod gui;
mod fps;
mod frametimes;

use unicorn;


use crate::{
    gui::{framework::Framework, Gui},
};


use log::{debug, error, log_enabled, info, Level};
use env_logger;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
    env,
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, Event, MouseScrollDelta, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;
use gilrs::Gilrs;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut uc = unicorn::core::Unicorn::new();

    let mut gilrs = Gilrs::new().unwrap();


    let event_loop = EventLoop::new();

    let window = init_window(&event_loop);
    let window_size = window.inner_size();
    let scale_factor = window.scale_factor() as f32;
    let mut pixels = init_pixels(&window);

    let mut input = WinitInputHelper::new();
    let mut last_update = Instant::now();
    
    let mut times = frametimes::FrameTimes::new(Duration::from_secs(1) / 60);
    let mut fps_counter = fps::FpsCounter::new();
    let mut previous_frame_time = Instant::now();

    times.reset();
    uc.setup();


    let mut framework = Framework::new(
        window_size.width,
        window_size.height,
        scale_factor,
        &pixels,
        Gui::default(),
        &event_loop,
    );

    event_loop.run(move |event, _, control_flow| {
        times.update();
        fps_counter.update(times.get_last_time());
        uc.fps = fps_counter.get_fps();

        if let Event::WindowEvent { event, .. } = &event {
            framework.handle_event(event);
        }

        framework.prepare(
            &mut pixels,
            &window,
            &mut uc,
            &mut gilrs,
        );

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Put in pause
            if input.key_pressed(VirtualKeyCode::F1) {
                framework.gui.window_open = !framework.gui.window_open;
                uc.switch_pause();
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                framework.resize(size.width, size.height);
            }
        
            /*
            screen.cls(5);

            screen.line(0, 0, 50, 50, 7);
            screen.trigon(0, 0, 50, 70, 100, 90, 4);
            screen.print("Hello World".to_string(), 64, 64, 6);
            */

            if uc.state == unicorn::core::UnicornState::RUN {
                uc.update();
                uc.draw();
            
                let now = Instant::now();
                let dt = now.duration_since(previous_frame_time);
                previous_frame_time = now;
                uc.update_time(dt);
            }

            let screen = &mut uc.screen.lock().unwrap();
            pixels.get_frame_mut().copy_from_slice(&screen.pixel_buffer);

            let render_result = pixels.render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                framework.render(encoder, render_target, context)?;
                Ok(())
            });

            if render_result.is_err() {
                println!("render_with failed");
                *control_flow = ControlFlow::Exit;
                return;
            }

            window.request_redraw();
            //times.limit();
        }

    });

}

const DEFAULT_WINDOW_RESOLUTION: unicorn::core::resolution::Resolution = unicorn::core::resolution::Resolution::High;

fn init_window(event_loop: &EventLoop<()>) -> Window {
    let size = LogicalSize::new(
        DEFAULT_WINDOW_RESOLUTION.width() as f64,
        DEFAULT_WINDOW_RESOLUTION.height() as f64,
    );
    WindowBuilder::new()
        .with_title("Unicorn Console")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(event_loop)
        .unwrap()
}


fn init_pixels(window: &Window) -> Pixels {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

    Pixels::new(128, 128, surface_texture).unwrap()
}