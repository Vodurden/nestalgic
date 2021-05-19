use super::Pixel;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Texture {
    pub pixels: Vec<Pixel>,
    pub width: usize,
    pub height: usize,
}

impl Texture {
    pub fn new(pixels: &[Pixel], width: usize, height: usize) -> Texture {
        assert!(
            pixels.len() == width * height,
            "not enough pixels for texture of size {}x{}, (pixels.len()={}, expected={})",
            width,
            height,
            pixels.len(),
            width * height
        );

        Texture {
            pixels: pixels.into(),
            width,
            height
        }
    }

    /// The NES stores pattern tables as bitplanes, which is a packed format that
    /// represents larger bytes as sequences of bits, which must be combined to form
    /// the true "byte".
    ///
    /// This function assumes we want to merge a bitplane with a bit depth of 2.
    ///
    /// # Arguments
    ///
    /// - `bytes`: The array of bytes containing bitplanes
    /// - `tile_length`: The number of bytes per tile
    ///
    /// # References
    ///
    /// - https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
    pub fn from_bitplanes(
        bytes: &[u8], tile_length: usize, width: usize, height: usize
    ) -> Texture {
        assert!(
            bytes.len() % tile_length == 0,
            "bytes length ({}) must be divisible by tile_length ({})",
            bytes.len(),
            tile_length
        );

        assert!(
            tile_length % 2 == 0,
            "tile_length ({}) must be divisible by 2",
            tile_length
        );

        // Each 16 bytes defines a 8x8 sprite within the pattern table, unfortunately there isn't a linear
        // relationship between bytes and pixels which means we need to translate from our byte indexes to
        // our target pixel coordinates.
        let mut pixels = vec![Pixel::empty(); width * height];
        for (i, chr) in bytes.chunks(16).enumerate() {
            for y in 0..8 {
                let line_byte_1 = chr[y];
                let line_byte_2 = chr[8 + y];

                for x in 0..8 {
                    let pixel_bit_1 = (line_byte_1 >> 7 - x) & 1;
                    let pixel_bit_2 = (line_byte_2 >> 7 - x) & 1;
                    let pixel_value = pixel_bit_1 + (pixel_bit_2 << 1);

                    let offset_x = (i * 8) % width;
                    let offset_y = (i / 16) * 8;
                    let pixel_x = offset_x + x;
                    let pixel_y = offset_y + y;

                    pixels[(pixel_y * width) + pixel_x] = match pixel_value {
                        0 => Pixel::empty(),
                        1 => Pixel::new(255, 0, 0, 255),
                        2 => Pixel::new(0, 255, 0, 255),
                        3 => Pixel::new(0, 0, 255, 255),
                        _ => Pixel::new(255, 0, 255, 255)
                    };
                }
            }
        }

        Texture::new(&pixels, width, height)
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|pixel| pixel.into_rgba().iter().cloned().collect::<Vec<u8>>())
            .collect()
    }

    pub fn render_ascii(&self) -> String {
        self.pixels
            .chunks(self.height)
            .map(|pixel_row| {
                let row_text = pixel_row.iter().map(|pixel| {
                    match pixel {
                        Pixel { red: 0, green: 0, blue: 0, alpha: 0 } => '.',
                        Pixel { red: 255, green: 0, blue: 0, alpha: 255 } => '1',
                        Pixel { red: 0, green: 255, blue: 0, alpha: 255 } => '2',
                        Pixel { red: 0, green: 0, blue: 255, alpha: 255 } => '3',
                        _ => '?'
                    }
                }).collect::<Vec<char>>();

                row_text
                    .chunks(8)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pixel;

    #[test]
    pub fn texture_from_bitplanes() {
        let bytes = vec![
            // Plane 1
            0b01000001,
            0b11000010,
            0b01000100,
            0b01001000,
            0b00010000,
            0b00100000,
            0b01000000,
            0b10000000,

            // Plane 2
            0b00000001,
            0b00000010,
            0b00000100,
            0b00001000,
            0b00010110,
            0b00100001,
            0b01000010,
            0b10000111,

            // Plane 1
            0b01000011,
            0b11000010,
            0b01000100,
            0b01001000,
            0b00010000,
            0b00100000,
            0b01000000,
            0b10000000,

            // Plane 2
            0b00000011,
            0b00000010,
            0b00000100,
            0b00001000,
            0b00010110,
            0b00100001,
            0b01000010,
            0b10000111,
        ];

        let expected = vec![
            0,1,0,0,0,0,0,3,
            1,1,0,0,0,0,3,0,
            0,1,0,0,0,3,0,0,
            0,1,0,0,3,0,0,0,
            0,0,0,3,0,2,2,0,
            0,0,3,0,0,0,0,2,
            0,3,0,0,0,0,2,0,
            3,0,0,0,0,2,2,2,

            0,1,0,0,0,0,3,3,
            1,1,0,0,0,0,3,0,
            0,1,0,0,0,3,0,0,
            0,1,0,0,3,0,0,0,
            0,0,0,3,0,2,2,0,
            0,0,3,0,0,0,0,2,
            0,3,0,0,0,0,2,0,
            3,0,0,0,0,2,2,2,
        ];
        let expected: Vec<Pixel> = expected.into_iter().map(|colour| {
            match colour {
                0 => Pixel::empty(),
                1 => Pixel::new(255, 0, 0, 255),
                2 => Pixel::new(0, 255, 0, 255),
                3 => Pixel::new(0, 0, 255, 255),
                _ => Pixel::new(255, 0, 255, 255)
            }
        }).collect();
        let expected = Texture::new(&expected, 16, 8);

        let result = Texture::from_bitplanes(&bytes, 16, 16, 8);

        assert_eq!(result, expected);
    }
}
