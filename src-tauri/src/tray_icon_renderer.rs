use tiny_skia::Pixmap;

#[derive(Clone, Debug)]
pub struct GlyphBitmap {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance: f32,
    pub alpha: Vec<u8>,
}

impl GlyphBitmap {
    pub fn new(
        width: usize,
        height: usize,
        xmin: i32,
        ymin: i32,
        advance: f32,
        alpha: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            xmin,
            ymin,
            advance,
            alpha,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DigitAtlas {
    pub font_px: f32,
    pub glyphs: [GlyphBitmap; 10],
}

impl DigitAtlas {
    pub fn from_glyphs(glyphs: [GlyphBitmap; 10], font_px: f32) -> Self {
        Self { font_px, glyphs }
    }

    pub fn from_font(font: &fontdue::Font, font_px: f32) -> Self {
        let glyphs = core::array::from_fn(|d| {
            let ch = char::from_digit(d as u32, 10).unwrap_or('0');
            let (metrics, alpha) = font.rasterize(ch, font_px);
            GlyphBitmap {
                width: metrics.width,
                height: metrics.height,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance: metrics.advance_width,
                alpha,
            }
        });
        Self { font_px, glyphs }
    }
}

#[derive(Clone, Debug)]
pub struct TrayImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl TrayImage {
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn into_tauri_image(self) -> tauri::image::Image<'static> {
        tauri::image::Image::new_owned(self.rgba, self.width, self.height)
    }
}

pub struct TrayIconRenderer {
    atlas: DigitAtlas,
}

impl TrayIconRenderer {
    pub fn new(atlas: DigitAtlas) -> Self {
        Self { atlas }
    }

    pub fn from_font_bytes(font_bytes: &[u8], font_px: f32) -> Result<Self, String> {
        let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .map_err(|err| err.to_string())?;
        let atlas = DigitAtlas::from_font(&font, font_px);
        Ok(Self { atlas })
    }

    pub fn render_text(&self, text: &str, size_px: u32) -> TrayImage {
        let mut pixmap = Pixmap::new(size_px, size_px).expect("pixmap");
        let rgba = pixmap.data_mut();

        for px in rgba.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
        }

        let mut pen_x: i32 = 1;
        let baseline: i32 = size_px as i32 - 3;

        for ch in text.chars() {
            let digit = match ch.to_digit(10) {
                Some(d) => d as usize,
                None => continue,
            };

            let glyph = &self.atlas.glyphs[digit];
            let glyph_w = glyph.width as i32;
            let glyph_h = glyph.height as i32;
            let glyph_x = pen_x + glyph.xmin;
            let glyph_y = baseline - glyph.ymin - glyph_h;

            for y in 0..glyph_h {
                let dst_y = glyph_y + y;
                if dst_y < 0 || dst_y >= size_px as i32 {
                    continue;
                }
                for x in 0..glyph_w {
                    let dst_x = glyph_x + x;
                    if dst_x < 0 || dst_x >= size_px as i32 {
                        continue;
                    }
                    let src_index = (y as usize * glyph.width) + x as usize;
                    let a = *glyph.alpha.get(src_index).unwrap_or(&0);
                    let dst_index = ((dst_y as u32 * size_px + dst_x as u32) * 4) as usize;
                    rgba[dst_index] = a;
                    rgba[dst_index + 1] = a;
                    rgba[dst_index + 2] = a;
                    rgba[dst_index + 3] = a;
                }
            }

            pen_x += glyph.advance.round() as i32;
        }

        TrayImage::new(pixmap.data().to_vec(), size_px, size_px)
    }

    pub fn render_with_icon(
        &self,
        text: &str,
        icon_rgba: &[u8],
        icon_width: u32,
        icon_height: u32,
        _percentage: f32, // Unused but kept for API compatibility
    ) -> TrayImage {
        // Canvas: icon (16px) + text (no circle)
        let icon_size: u32 = 16;
        let padding: u32 = 2;
        let text_width = estimate_text_width(text);
        let total_width = icon_size + padding + text_width;
        let height = icon_size;

        let mut pixmap = Pixmap::new(total_width, height).expect("pixmap");
        let rgba = pixmap.data_mut();

        // Clear to transparent
        for px in rgba.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
        }

        // Draw icon on the left (simple copy since icon should already be 16x16)
        for y in 0..icon_height.min(16) {
            for x in 0..icon_width.min(16) {
                let src_idx = ((y * icon_width + x) * 4) as usize;
                let dst_idx = ((y * total_width + x) * 4) as usize;

                if src_idx + 4 <= icon_rgba.len() && dst_idx + 4 <= rgba.len() {
                    // Only copy non-transparent pixels
                    let alpha = icon_rgba[src_idx + 3];
                    if alpha > 0 {
                        rgba[dst_idx..dst_idx + 4]
                            .copy_from_slice(&icon_rgba[src_idx..src_idx + 4]);
                    }
                }
            }
        }

        // Draw text
        let mut text_x = icon_size + padding;
        let baseline = height as i32 - 3;

        for ch in text.chars() {
            let digit = match ch.to_digit(10) {
                Some(d) => d as usize,
                None => continue,
            };

            let glyph = &self.atlas.glyphs[digit];
            let glyph_w = glyph.width as i32;
            let glyph_h = glyph.height as i32;
            let glyph_x = text_x as i32 + glyph.xmin;
            let glyph_y = baseline - glyph.ymin - glyph_h;

            for y in 0..glyph_h {
                let dst_y = glyph_y + y;
                if dst_y < 0 || dst_y >= height as i32 {
                    continue;
                }
                for x in 0..glyph_w {
                    let dst_x = glyph_x + x;
                    if dst_x < 0 || dst_x >= total_width as i32 {
                        continue;
                    }
                    let src_index = (y as usize * glyph.width) + x as usize;
                    let a = *glyph.alpha.get(src_index).unwrap_or(&0);
                    let dst_index = ((dst_y as u32 * total_width + dst_x as u32) * 4) as usize;
                    rgba[dst_index] = a;
                    rgba[dst_index + 1] = a;
                    rgba[dst_index + 2] = a;
                    rgba[dst_index + 3] = a;
                }
            }

            text_x += glyph.advance.round() as u32;
        }

        TrayImage::new(pixmap.data().to_vec(), total_width, height)
    }
}

fn estimate_text_width(text: &str) -> u32 {
    // Rough estimate: ~7 pixels per digit
    text.len() as u32 * 7
}
