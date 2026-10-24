//! The vanilla Minecraft bitmap font (`assets/minecraft/textures/font/ascii.png`): a 16×16 grid of 8×8 cells, one
//! per code point below 256. A glyph's advance is its rightmost lit column + 1, plus one gap column; a space is 3.

pub struct Font {
    lit: Vec<bool>, // 128×128
}

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub width: usize,
    pub rows: [[bool; 8]; 8],
}

/// A binary image, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub bits: Vec<bool>,
}

impl Bitmap {
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.bits[y * self.width + x]
    }
    pub fn lit(&self) -> usize {
        self.bits.iter().filter(|&&b| b).count()
    }
    /// The sub-rectangle at (x, y) of the given size.
    pub fn crop(&self, x: usize, y: usize, width: usize, height: usize) -> Bitmap {
        let bits = (0..height).flat_map(|dy| (0..width).map(move |dx| (dx, dy))).map(|(dx, dy)| self.get(x + dx, y + dy)).collect();
        Bitmap { width, height, bits }
    }
}

impl Font {
    pub fn load(png_bytes: &[u8]) -> Result<Font, String> {
        let decoder = png::Decoder::new(png_bytes);
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
        if (info.width, info.height) != (128, 128) {
            return Err(format!("ascii.png is {}×{}, expected 128×128", info.width, info.height));
        }
        let channels = match info.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::GrayscaleAlpha => 2,
            other => return Err(format!("ascii.png colour type {other:?} has no alpha channel")),
        };
        let lit = (0..128 * 128).map(|i| buf[i * channels + channels - 1] > 0).collect();
        Ok(Font { lit })
    }
}

impl Font {
    /// Reads the sheet from `path`. The file is Mojang's and is not shipped with this tool.
    pub fn from_file(path: &std::path::Path) -> Result<Font, String> {
        let how = "get it from your Minecraft client jar (a .jar is a zip: assets/minecraft/textures/font/ascii.png) and pass its path in HUD_CADENCE_FONT, or put it at ./ascii.png";
        let bytes = std::fs::read(path).map_err(|e| format!("cannot read the font sheet {} ({e}); {how}", path.display()))?;
        Font::load(&bytes).map_err(|e| format!("{} is not a usable font sheet: {e}; {how}", path.display()))
    }

    /// The font from `HUD_CADENCE_FONT`, else `ascii.png` or `assets/ascii.png` in the current directory.
    pub fn locate() -> Result<Font, String> {
        if let Some(p) = std::env::var_os("HUD_CADENCE_FONT") {
            return Font::from_file(std::path::Path::new(&p));
        }
        let local = ["ascii.png", "assets/ascii.png"].into_iter().map(std::path::Path::new).find(|p| p.exists());
        Font::from_file(local.unwrap_or(std::path::Path::new("ascii.png")))
    }
}

/// Glyph for `ch` (code points below 256). `width` is the advance before the gap column.
pub fn glyph(font: &Font, ch: char) -> Glyph {
    let code = ch as usize;
    let (row, col) = (code / 16, code % 16);
    let mut rows = [[false; 8]; 8];
    let mut width = 0;
    for y in 0..8 {
        for x in 0..8 {
            let on = font.lit[(row * 8 + y) * 128 + col * 8 + x];
            rows[y][x] = on;
            if on {
                width = width.max(x + 1);
            }
        }
    }
    if ch == ' ' {
        width = 3;
    }
    Glyph { width, rows }
}

/// `text` as Minecraft lays it out, each font pixel a `scale`×`scale` square, a one-font-pixel gap after each glyph.
pub fn render(font: &Font, text: &str, scale: usize) -> Bitmap {
    let glyphs: Vec<Glyph> = text.chars().map(|c| glyph(font, c)).collect();
    let font_w: usize = glyphs.iter().map(|g| g.width + 1).sum();
    let (width, height) = (font_w * scale, 8 * scale);
    let mut bits = vec![false; width * height];
    let mut x0 = 0;
    for g in &glyphs {
        for y in 0..8 {
            for x in 0..g.width {
                if g.rows[y][x] {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            bits[(y * scale + dy) * width + (x0 + x) * scale + dx] = true;
                        }
                    }
                }
            }
        }
        x0 += g.width + 1;
    }
    Bitmap { width, height, bits }
}
