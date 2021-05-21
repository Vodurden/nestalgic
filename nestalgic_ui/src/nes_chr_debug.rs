use imgui::{Condition, Image, StyleVar::WindowPadding, TextureId, Ui, im_str};
use imgui_wgpu::{Renderer, Texture, TextureConfig};
use nestalgic::Nestalgic;
use wgpu::{Device, Extent3d, Queue};
use crate::ext::imgui_wgpu::TextureExt;

pub struct NesChrDebug {
    chr_texture_id: TextureId
}

const WIDTH: usize = 128;
const HEIGHT: usize = 128;
const DEFAULT_SCALE: usize = 6;

impl NesChrDebug {
    pub fn new(device: &Device, renderer: &mut Renderer) -> NesChrDebug {
        let texture_config = TextureConfig {
            size: Extent3d {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                ..Default::default()
            },
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            label: Some("nes chr debug texture"),
            ..Default::default()
        };

        let chr_texture = Texture::new_with_nearest_scaling(&device, texture_config);
        let chr_texture_id = renderer.textures.insert(chr_texture);

        NesChrDebug {
            chr_texture_id
        }
    }

    pub fn render(
        &mut self,
        ui: &Ui,
        nestalgic: &Nestalgic,
        wgpu_queue: &Queue,
        imgui_renderer: &mut Renderer
    ) {
        let window = imgui::Window::new(im_str!("Nes CHR Debug"));

        let nes_texture = nestalgic.pattern_table();
        if let Some(chr_texture) = imgui_renderer.textures.get(self.chr_texture_id) {
            let wgpu_texture_data = nes_texture.to_rgba();
            chr_texture.write(&wgpu_queue, &wgpu_texture_data, WIDTH as u32, HEIGHT as u32);
        }

        let style = ui.push_style_var(WindowPadding([10.0, 10.0]));

        window
            .size([(WIDTH * DEFAULT_SCALE) as f32, (WIDTH * DEFAULT_SCALE) as f32], Condition::FirstUseEver)
            .build(&ui, || {
                let window_size = ui.window_size();
                let content_region = ui.content_region_avail();
                let smallest_dimension = content_region[0].min(content_region[1]);
                let image_width = [smallest_dimension; 2];

                let image_position = [
                    (content_region[0] - image_width[0]) * 0.5 + (window_size[0] - content_region[0]) * 0.5,
                    ui.cursor_pos()[1]
                ];

                ui.set_cursor_pos(image_position);

                Image::new(self.chr_texture_id, image_width).build(&ui);

            });

        style.pop(ui);
    }
}
