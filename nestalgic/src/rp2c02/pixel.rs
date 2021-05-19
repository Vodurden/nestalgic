#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Pixel {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Pixel {
        Pixel { red, green, blue, alpha }
    }

    pub fn empty() -> Pixel {
        Pixel::new(0, 0, 0, 0)
    }

    pub fn into_rgba(&self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    pub fn into_texture(pixels: &[Pixel]) -> Vec<u8> {
        pixels
            .into_iter()
            .flat_map(|pixel| pixel.into_rgba().iter().cloned().collect::<Vec<u8>>())
            .collect()
    }
}
