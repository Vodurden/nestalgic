use imgui::{im_str, Ui, Image, TextureId, Condition};
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use wgpu::{Queue, Device, Extent3d};

pub struct NesChrDebug {
    chr_texture_id: TextureId
}

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

impl NesChrDebug {
    pub fn new(device: &Device, queue: &Queue, renderer: &mut Renderer) -> NesChrDebug {
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

        // `pixels` is RGBA so 4 bytes per pixel
        let pixels: [u8; WIDTH * HEIGHT * 4] = [100; WIDTH * HEIGHT * 4];
        chr_texture.write(&queue, &pixels, WIDTH as u32, HEIGHT as u32);
        let chr_texture_id = renderer.textures.insert(chr_texture);

        NesChrDebug {
            chr_texture_id
        }
    }

    pub fn render(&mut self, ui: &Ui) {
        let window = imgui::Window::new(im_str!("Nes CHR Debug"));

        window
            .size([WIDTH as f32, HEIGHT as f32], Condition::FirstUseEver)
            .build(&ui, || {
                Image::new(self.chr_texture_id, [WIDTH as f32, HEIGHT as f32]).build(&ui);
            });
    }
}
