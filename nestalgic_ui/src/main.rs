#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod ui;
mod nes_texture_window;
mod nestalgic_ui;
mod ext;

use anyhow::{Result, Context};
use log::error;
use nestalgic::{NESROM, Nestalgic};
use nestalgic_ui::NestalgicUI;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 960;

fn main() -> Result<()> {
    env_logger::init();

    let rom_file = include_bytes!("../../roms/donkey-kong.nes").to_vec();
    let rom = NESROM::from_bytes(rom_file).context("Failed to load ROM")?;
    let nestalgic = Nestalgic::new(rom);

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

    let mut nestalgic_ui = NestalgicUI::new(nestalgic, &window)
        .context("Could not create NestalgicUI")?;

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            if let Err(error) = nestalgic_ui.render(&window) {
                error!("render failed: {}", error);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        nestalgic_ui.handle_event(&window, &event);
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            nestalgic_ui.update(&input);

            window.request_redraw();
        }
    });
}
