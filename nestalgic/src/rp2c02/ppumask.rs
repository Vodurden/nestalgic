/// `PPUMask` represents the PPU control register mapped to `0x2001`
///
/// Each bit in `PPUMask` has a different meaning:
///
/// ```text
/// +---+---+---+---+---+---+---+---+
/// | B | G | R | s | b | M | m | G |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   |
///   |   |   |   |   |   |   |   \-------- Greyscale
///   |   |   |   |   |   |   |             (0: normal colour, 1: produce greyscale display)
///   |   |   |   |   |   |   |
///   |   |   |   |   |   |   \------------ Show background in leftmost 8 pixels of the screen
///   |   |   |   |   |   |                 (0: hide, 1: show)
///   |   |   |   |   |   |
///   |   |   |   |   |   \---------------- Show sprites in leftmost 8 pixels of the screen
///   |   |   |   |   |                     (0: hide, 1: show)
///   |   |   |   |   |
///   |   |   |   |   \-------------------- Sprite pattern table address for 8x8 sprites
///   |   |   |   |                         (0: 0x0000, 1: 0x1000. Ignored in 8x16 mode)
///   |   |   |   |
///   |   |   |   \------------------------ Show backgrounds
///   |   |   |                             (0: hide, 1: show)
///   |   |   |
///   |   |   \---------------------------- Emphasise red
///   |   |
///   |   \-------------------------------- Emphasise green
///   |
///   \------------------------------------ Emphasise blue
/// ```
///
/// See also: https://wiki.nesdev.com/w/index.php/PPU_registers
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct PPUMask {
    /// Force the palette to only use colours from the grey column (0x00, 0x10, 0x20 and 0x30).
    pub greyscale: bool,

    pub show_background_on_left_8_pixels: bool,

    pub show_sprites_on_left_8_pixels: bool,

    pub show_background: bool,

    pub show_sprites: bool,

    pub emphasise_red: bool,

    pub emphasise_green: bool,

    pub emphasise_blue: bool,
}

impl Default for PPUMask {
    fn default() -> Self {
        0.into()
    }
}

impl From<u8> for PPUMask {
    fn from(byte: u8) -> Self {
        let greyscale =                        byte & 0b0000_0001 != 0;
        let show_background_on_left_8_pixels = byte & 0b0000_0010 != 0;
        let show_sprites_on_left_8_pixels =    byte & 0b0000_0100 != 0;
        let show_background =                  byte & 0b0000_1000 != 0;
        let show_sprites =                     byte & 0b0001_0000 != 0;
        let emphasise_red =                    byte & 0b0010_0000 != 0;
        let emphasise_green =                  byte & 0b0100_0000 != 0;
        let emphasise_blue =                   byte & 0b1000_0000 != 0;

        PPUMask {
            greyscale,
            show_background_on_left_8_pixels,
            show_sprites_on_left_8_pixels,
            show_background,
            show_sprites,
            emphasise_red,
            emphasise_green,
            emphasise_blue,
        }
    }
}

impl From<PPUMask> for u8 {
    fn from(mask: PPUMask) -> Self {
        let greyscale =                        (mask.greyscale as u8) << 0;
        let show_background_on_left_8_pixels = (mask.show_background_on_left_8_pixels as u8) << 1;
        let show_sprites_on_left_8_pixels =    (mask.show_sprites_on_left_8_pixels as u8) << 2;
        let show_background =                  (mask.show_background as u8) << 3;
        let show_sprites =                     (mask.show_sprites as u8) << 4;
        let emphasise_red =                    (mask.emphasise_red as u8) << 5;
        let emphasise_green =                  (mask.emphasise_green as u8) << 6;
        let emphasise_blue =                   (mask.emphasise_blue as u8) << 7;

        greyscale
            | show_background_on_left_8_pixels
            | show_sprites_on_left_8_pixels
            | show_background
            | show_sprites
            | emphasise_red
            | emphasise_green
            | emphasise_blue
    }
}
