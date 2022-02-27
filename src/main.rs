mod first_app;

use first_app::*;

use winit::{
    dpi::{PhysicalSize},
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    event_loop::ControlFlow,
};

use std::time::Instant;

fn main() {
    env_logger::init();

    let ( mut vulkan_app, event_loop) = VulkanApp::new();

    log::debug!("Running Application");

    let mut current_time = Instant::now();

    let mut keys_pressed: Vec<VirtualKeyCode> = Vec::new();

    event_loop.run(move |event, _, control_flow| {
        
        *control_flow = ControlFlow::Poll;
        
        let app = &mut vulkan_app;

        app.platform.handle_event(app.imgui.io_mut(), &app.window, &event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::debug!("Closing window");
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(PhysicalSize { width, height }),
                ..
            } => {
                log::debug!("Resizing window");
                log::info!("New window size: {}x{}", width, height);
                app.resize();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                match input.virtual_keycode {
                    /*Some(VirtualKeyCode::Escape) => {
                        log::debug!("Closing window");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }*/
                    Some(input_key) => {
                        match input.state {
                            ElementState::Pressed => {
                                if !keys_pressed.contains(&input_key) {
                                    keys_pressed.push(input_key);
                                }
                            }
                            ElementState::Released => {
                                let index = keys_pressed
                                .iter()
                                .position(|key| *key == input_key)
                                .unwrap();
                                keys_pressed.remove(index);
                            }
                        };
                    }
                    None => {}
                };
            }
            Event::MainEventsCleared => {
                app.window.request_redraw();
            },
            Event::RedrawRequested(_window_id) => {
                let frame_time = current_time.elapsed().as_secs_f32();
                app.run(&keys_pressed, frame_time);
                current_time = Instant::now();
            }
            _ => (),
        };

    });
}
