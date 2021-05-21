use std::time::Duration;

use anyhow::{Result, Context};
use nestalgic::Nestalgic;

use crate::nes_chr_debug::NesChrDebug;

pub struct UI {
    imgui: imgui::Context,
    imgui_platform: imgui_winit_support::WinitPlatform,
    imgui_renderer: imgui_wgpu::Renderer,

    nes_chr_debug: NesChrDebug,
}

impl UI {
    pub fn new(
        window: &winit::window::Window,
        wgpu_device: &wgpu::Device,
        wgpu_queue: &wgpu::Queue,
    ) -> UI {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        imgui_platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Fix incorrect colors with sRGB framebuffer
        let style = imgui.style_mut();
        for color in 0..style.colors.len() {
            style.colors[color] = gamma_to_linear(style.colors[color]);
        }

        let texture_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let config = imgui_wgpu::RendererConfig {
            texture_format,
            ..Default::default()
        };
        let mut imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui, wgpu_device, wgpu_queue, config
        );

        let nes_chr_debug = NesChrDebug::new(wgpu_device, &mut imgui_renderer);

        UI {
            imgui,
            imgui_platform,
            imgui_renderer,

            nes_chr_debug,
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.imgui_platform.handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn update(&mut self, delta: Duration) {
        self.imgui.io_mut().update_delta_time(delta);
    }

    pub fn prepare(&mut self, window: &winit::window::Window) -> Result<()> {
        self.imgui_platform.prepare_frame(self.imgui.io_mut(), window)
            .context("Could not prepare UI")
    }

    pub fn render(
        &mut self,
        nestalgic: &Nestalgic,
        render_target: &wgpu::TextureView,
        wgpu_encoder: &mut wgpu::CommandEncoder,
        wgpu_queue: &wgpu::Queue,
        wgpu_device: &wgpu::Device
    ) -> Result<()> {
        let ui = self.imgui.frame();

        self.nes_chr_debug.render(&ui, nestalgic, wgpu_queue, &mut self.imgui_renderer);

        // Render Dear ImGui with WGPU
        let mut rpass = wgpu_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.imgui_renderer
            .render(ui.render(), wgpu_queue, wgpu_device, &mut rpass)
            .context("imgui render failed")
    }
}

fn gamma_to_linear(color: [f32; 4]) -> [f32; 4] {
    const GAMMA: f32 = 2.2;

    let x = color[0].powf(GAMMA);
    let y = color[1].powf(GAMMA);
    let z = color[2].powf(GAMMA);
    let w = 1.0 - (1.0 - color[3]).powf(GAMMA);

    [x, y, z, w]
}
