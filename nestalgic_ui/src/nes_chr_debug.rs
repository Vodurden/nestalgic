use imgui::{Condition, Image, TextureId, Ui, im_str};
use imgui_wgpu::{Renderer, Texture, TextureConfig};
use nestalgic::Nestalgic;
use wgpu::{Queue, Device, Extent3d};

pub struct NesChrDebug {
    chr_texture_id: TextureId
}

const WIDTH: usize = 128;
const HEIGHT: usize = 128;

impl NesChrDebug {
    pub fn new(device: &Device, renderer: &mut Renderer) -> NesChrDebug {
        let texture_config = TextureConfig {
            size: Extent3d {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                ..Default::default()
            },
            label: Some("nes chr debug texture"),
            ..Default::default()
        };

        let chr_texture = Texture::new(&device, &renderer, texture_config);
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
        wgpu_renderer: &mut Renderer
    ) {
        let window = imgui::Window::new(im_str!("Nes CHR Debug"));

        let nes_texture = nestalgic.pattern_table();
        if let Some(chr_texture) = wgpu_renderer.textures.get(self.chr_texture_id) {
            let wgpu_texture_data = nes_texture.to_rgba();
            chr_texture.write(&wgpu_queue, &wgpu_texture_data, WIDTH as u32, HEIGHT as u32);
        }

        window
            .size([WIDTH as f32, HEIGHT as f32], Condition::FirstUseEver)
            .build(&ui, || {
                Image::new(self.chr_texture_id, [(WIDTH * 4) as f32, (HEIGHT * 4) as f32]).build(&ui);
            });
    }
}
