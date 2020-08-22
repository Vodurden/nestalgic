#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use nestalgic::{Nestalgic, NESROM};
use std::time::Instant;

const WIDTH: u32 = Nestalgic::SCREEN_WIDTH as u32;
const HEIGHT: u32 = Nestalgic::SCREEN_HEIGHT as u32;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    nestalgic: Nestalgic,
    time_of_last_update: Instant,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Nestalgic")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> World {
        let rom_file = include_bytes!("../../roms/donkey-kong.nes").to_vec();
        let rom = NESROM::from_bytes(rom_file).expect("Failed to load ROM");
        let nestalgic = Nestalgic::new().with_rom(rom);

        World {
            nestalgic,
            time_of_last_update: Instant::now(),
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(self.time_of_last_update);

        self.nestalgic.tick(delta_time);

        self.time_of_last_update = current_time;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        let nestalgic_pixels = self.nestalgic.pixels();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = nestalgic_pixels[i].into_rgba();
            pixel.copy_from_slice(&rgba);
        }
    }
}
