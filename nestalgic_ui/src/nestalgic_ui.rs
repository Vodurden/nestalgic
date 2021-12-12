use std::time::Instant;

use nestalgic::Nestalgic;
use pixels::{Pixels, SurfaceTexture};

use anyhow::{Result, Context};
use winit_input_helper::WinitInputHelper;

use crate::ui::UI;

pub struct NestalgicUI {
    nestalgic: Nestalgic,

    time_of_last_update: Instant,
    scale_factor: f64,

    ui: UI,

    pixels: Pixels
}

impl NestalgicUI {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    pub fn new(
        nestalgic: Nestalgic,
        window: &winit::window::Window
    ) -> Result<NestalgicUI> {
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
            Pixels::new(NestalgicUI::WIDTH, NestalgicUI::HEIGHT, surface_texture)
                .context("Could not create pixels surface")?
        };

        let ui = UI::new(window, pixels.device(), pixels.queue());

        Ok(NestalgicUI {
            nestalgic,
            time_of_last_update: Instant::now(),
            scale_factor: window.scale_factor(),
            ui,
            pixels,
        })
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>
    ) {
        self.ui.handle_event(window, event);
    }

    pub fn update(&mut self, input: &WinitInputHelper) {
        let now = Instant::now();
        let delta = now - self.time_of_last_update;
        self.time_of_last_update = now;

        if let Some(scale_factor) = input.scale_factor() {
            self.scale_factor = scale_factor;
        }

        if let Some(size) = input.window_resized() {
            self.pixels.resize_surface(size.width, size.height);

            // TODO: Do we need this?
            // Resize the world
            // let LogicalSize { width, height } = size.to_logical(scale_factor);
            // world.resize(width, height);
            // pixels.resize_buffer(width, height);
        }

        self.ui.update(delta);
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<()> {
        let frame = self.pixels.get_frame();
        NestalgicUI::render_nes(&self.nestalgic, frame);

        self.ui.prepare(window)?;

        let nestalgic = &self.nestalgic;
        let ui = &mut self.ui;
        self.pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);

            ui.render(
                nestalgic,
                render_target,
                encoder,
                &context.queue,
                &context.device
            ).expect("failed to render imgui");

            Ok(())
        })?;

        Ok(())
    }

    fn render_nes(_nestalgic: &Nestalgic, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            let rgba = [0x48, 0xb2, 0xe8, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}
