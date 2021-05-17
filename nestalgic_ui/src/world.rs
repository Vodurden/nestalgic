/// Representation of the application state. In this example, a box will bounce around the screen.
///
/// The world is resizable, meaning the backing pixel buffer can be resized without creating a
/// border around the screen.
pub struct World {
    width: i16,
    height: i16,
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as i16,
            height: height as i16,
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    pub fn update(&mut self) {
        if self.box_x <= 0 {
            self.velocity_x = 1;
        }
        if self.box_x > self.width {
            self.velocity_x = -1;
        }
        if self.box_y <= 0 {
            self.velocity_y = 1;
        }
        if self.box_y > self.height {
            self.velocity_y = -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Resize the world
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as i16;
        self.height = height as i16;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.width as usize) as i16;
            let y = (i / self.width as usize) as i16;

            let rgba = [0x48, 0xb2, 0xe8, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}
