use anyhow::{self, Context};
use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;
use winit_input_helper::WinitInputHelper;
use nestalgic::Nestalgic;

// pub struct NesDisplay {
//     pixels: Pixels
// }

// impl NesDisplay {
//     // pub fn new(window: &Window) -> anyhow::Result<NesDisplay> {
//     //     let window_size = window.inner_size();
//     //     let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
//     //     let pixels = Pixels::new(window_size.width, window_size.height, surface_texture)
//     //         .context("Could not create Pixel surface")?;

//     //     Ok(NesDisplay { pixels })
//     // }

//     pub fn handle_input(&mut self, input: &WinitInputHelper) {
//         if let Some(size) = input.window_resized() {
//             self.pixels.resize_surface(size.width, size.height);
//         }
//     }

//     pub fn render(pixels: &mut Pixels, nestalgic: &Nestalgic) -> anyhow::Result<()> {
//         let nestalgic_pixels = nestalgic.pixels();

//         for (i, pixel) in self.pixels.get_frame().chunks_exact_mut(4).enumerate() {
//             let rgba = nestalgic_pixels[i].into_rgba();
//             pixel.copy_from_slice(&rgba);
//         }

//         self.pixels.render().context("Could not render pixels")
//     }
// }
