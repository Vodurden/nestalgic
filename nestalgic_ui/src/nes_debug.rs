use std::time::{Duration, Instant};
use pixels::Pixels;
use winit::window::Window;
use winit::event::{Event, VirtualKeyCode};

// pub struct NesDebug {
//     imgui: imgui::Context,
//     platform: imgui_winit_support::WinitPlatform,
//     renderer: imgui_wgpu::Renderer,
//     last_frame: Instant,
// }

// impl NesDebug {
//     pub fn new(window: &winit::window::Window, pixels: &Pixels) -> Self {
//         // Create Dear ImGui context
//         let mut imgui = imgui::Context::create();
//         imgui.set_ini_filename(None);

//         // Initialize winit platform support
//         let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
//         platform.attach_window(
//             imgui.io_mut(),
//             &window,
//             imgui_winit_support::HiDpiMode::Default,
//         );

//         // Configure Dear ImGui fonts
//         let hidpi_factor = window.scale_factor();
//         let font_size = (13.0 * hidpi_factor) as f32;
//         imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
//         imgui
//             .fonts()
//             .add_font(&[imgui::FontSource::DefaultFontData {
//                 config: Some(imgui::FontConfig {
//                     oversample_h: 1,
//                     pixel_snap_h: true,
//                     size_pixels: font_size,
//                     ..Default::default()
//                 }),
//             }]);

//         // Fix incorrect colors with sRGB framebuffer
//         let style = imgui.style_mut();
//         for color in 0..style.colors.len() {
//             style.colors[color] = Self::gamma_to_linear(style.colors[color]);
//         }

//         // Create Dear ImGui WGPU renderer
//         let device = &pixels.context().device;
//         let queue = &pixels.context().queue;
//         let renderer_config = imgui_wgpu::RendererConfig {
//             texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
//             ..Default::default()
//         };
//         let renderer = imgui_wgpu::Renderer::new(&mut imgui, device, queue, renderer_config);
//         let mut last_frame = Instant::now();

//         NesDebug {
//             imgui,
//             platform,
//             renderer,
//             last_frame
//         }
//     }

//     pub fn handle_event(
//         &mut self,
//         pixels: Pixels,
//         window: &winit::window::Window,
//         event: Event<()>
//     ) -> anyhow::Result<()> {
//         match event {
//             Event::NewEvents(_) => {
//                 // other application-specific logic
//                 let now = Instant::now();
//                 self.imgui.io_mut().update_delta_time(now.duration_since(self.last_frame));
//                 self.last_frame = now;
//             },
//             Event::MainEventsCleared => {
//                 // other application-specific logic
//                 self.platform.prepare_frame(self.imgui.io_mut(), &window)
//                     .expect("Failed to prepare frame");
//             },
//             Event::RedrawRequested(_) => {
//                 let ui = self.imgui.frame();
//                 self.platform.prepare_render(&ui, &window);

//                 ui.text(im_str!("Hello world!"));

//                 let draw_data = ui.render();

//                 let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                     label: Some("imgui"),
//                     color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
//                         attachment: render_target,
//                         resolve_target: None,
//                         ops: wgpu::Operations {
//                             load: wgpu::LoadOp::Load,
//                             store: true,
//                         },
//                     }],
//                     depth_stencil_attachment: None,
//                 });

//                 self.renderer
//                     .render(draw_data, &pixels.context().queue, &pixels.context().device, &mut rpass)
//                     .expect("UI Rendering failed");
//             },
//             event => {
//                 self.platform.handle_event(self.imgui.io_mut(), &window, &event);
//             }
//         }

//         Ok(())
//     }

//     fn gamma_to_linear(color: [f32; 4]) -> [f32; 4] {
//         const GAMMA: f32 = 2.2;

//         let x = color[0].powf(GAMMA);
//         let y = color[1].powf(GAMMA);
//         let z = color[2].powf(GAMMA);
//         let w = 1.0 - (1.0 - color[3]).powf(GAMMA);

//         [x, y, z, w]
//     }
// }
