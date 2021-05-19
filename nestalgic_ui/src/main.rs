#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod gui;
mod world;
mod nes_chr_debug;

use crate::gui::Gui;
use crate::world::World;
use log::error;
use nestalgic::{NESROM, Nestalgic};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Error> {
    env_logger::init();

    let rom_file = include_bytes!("../../roms/donkey-kong.nes").to_vec();
    let rom = NESROM::from_bytes(rom_file).expect("Failed to load ROM");
    let nestalgic = Nestalgic::new().with_rom(rom);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + Dear ImGui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut scale_factor = window.scale_factor();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new(WIDTH, HEIGHT);

    // Set up Dear ImGui
    let mut gui = Gui::new(&window, &pixels);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Draw the world
            world.draw(pixels.get_frame());

            // Prepare Dear ImGui
            gui.prepare(&window).expect("gui.prepare() failed");

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render Dear ImGui
                gui.render(&window, encoder, render_target, context, &nestalgic)
                    .expect("gui.render() failed");
            });

            // Basic error handling
            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        gui.handle_event(&window, &event);
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(factor) = input.scale_factor() {
                scale_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                // Resize the surface texture
                pixels.resize_surface(size.width, size.height);

                // Resize the world
                let LogicalSize { width, height } = size.to_logical(scale_factor);
                world.resize(width, height);
                pixels.resize_buffer(width, height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}
