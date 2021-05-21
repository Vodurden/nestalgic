use imgui::{Condition, ImString, Image, StyleVar::WindowPadding, TextureId, Ui};
use imgui_wgpu::{Renderer, Texture, TextureConfig};
use nestalgic::Nestalgic;
use wgpu::{Device, Extent3d, Queue};
use crate::ext::imgui_wgpu::TextureExt;

pub struct NesTextureWindow {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub default_scale: usize,

    pub open: bool,

    get_nes_texture: fn(&Nestalgic) -> nestalgic::Texture,

    texture_id: TextureId
}

impl NesTextureWindow {
    pub fn new_chr_left_window(
        device: &Device,
        renderer: &mut Renderer
    ) -> NesTextureWindow {
        NesTextureWindow::new(
            device,
            renderer,
            "CHR Left",
            128,
            128,
            6,
            |nestalgic| nestalgic.pattern_table_left()
        )
    }

    pub fn new_chr_right_window(
        device: &Device,
        renderer: &mut Renderer
    ) -> NesTextureWindow {
        NesTextureWindow::new(
            device,
            renderer,
            "CHR Right",
            128,
            128,
            6,
            |nestalgic| nestalgic.pattern_table_right()
        )
    }

    pub fn new(
        device: &Device,
        renderer: &mut Renderer,
        name: &str,
        width: usize,
        height: usize,
        default_scale: usize,
        get_nes_texture: fn(&Nestalgic) -> nestalgic::Texture
    ) -> NesTextureWindow {
        let texture_config = TextureConfig {
            size: Extent3d {
                width: width as u32,
                height: height as u32,
                ..Default::default()
            },
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            label: Some(name),
            ..Default::default()
        };

        let texture = Texture::new_with_nearest_scaling(&device, texture_config);
        let texture_id = renderer.textures.insert(texture);

        NesTextureWindow {
            name: name.to_string(),
            width,
            height,
            default_scale,
            get_nes_texture,
            open: false,
            texture_id
        }
    }

    pub fn render(
        &mut self,
        ui: &Ui,
        nestalgic: &Nestalgic,
        wgpu_queue: &Queue,
        imgui_renderer: &mut Renderer
    ) {
        if !self.open { return; }

        let window_name = ImString::new(&self.name);
        let window = imgui::Window::new(&window_name);

        let nes_texture = (self.get_nes_texture)(nestalgic);
        if let Some(chr_texture) = imgui_renderer.textures.get(self.texture_id) {
            let wgpu_texture_data = nes_texture.to_rgba();
            chr_texture.write(&wgpu_queue, &wgpu_texture_data, self.width as u32, self.height as u32);
        }

        let style = ui.push_style_var(WindowPadding([10.0, 10.0]));

        let texture_id = self.texture_id;
        window
            .size([(self.width * self.default_scale) as f32, (self.width * self.default_scale) as f32], Condition::FirstUseEver)
            .opened(&mut self.open)
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

                Image::new(texture_id, image_width).build(&ui);

            });

        style.pop(ui);
    }
}
