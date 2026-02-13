use tiny_skia::Pixmap;

/// Default scale factor for Retina displays (2x for crisp rendering)
/// Can be overridden for Windows high DPI scenarios
const DEFAULT_SCALE_FACTOR: u32 = 2;

#[derive(Clone, Debug)]
pub struct TrayImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl TrayImage {
    /// Creates a new TrayImage with the specified RGBA pixel data and dimensions
    ///
    /// # Arguments
    /// * `rgba` - Vec of RGBA pixel data (4 bytes per pixel)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    /// Converts the TrayImage into a Tauri Image for use in tray icons
    ///
    /// Consumes self and transfers ownership of pixel data to the Tauri Image
    pub fn into_tauri_image(self) -> tauri::image::Image<'static> {
        tauri::image::Image::new_owned(self.rgba, self.width, self.height)
    }
}

pub struct TrayIconRenderer {
    font: fontdue::Font,
    font_px: f32,
    scale_factor: u32,
}

impl TrayIconRenderer {
    /// Creates a new TrayIconRenderer from font bytes with default scale factor
    ///
    /// # Arguments
    /// * `font_bytes` - Raw bytes of the TrueType/OpenType font
    /// * `font_px` - Font size in pixels for rendering
    ///
    /// # Returns
    /// Result containing the renderer or an error string
    pub fn from_font_bytes(font_bytes: &[u8], font_px: f32) -> Result<Self, String> {
        Self::from_font_bytes_with_scale(font_bytes, font_px, DEFAULT_SCALE_FACTOR)
    }

    /// Creates a new TrayIconRenderer from font bytes with custom scale factor
    ///
    /// # Arguments
    /// * `font_bytes` - Raw bytes of the TrueType/OpenType font
    /// * `font_px` - Font size in pixels for rendering
    /// * `scale_factor` - Scaling multiplier for high-DPI displays (2 for Retina, 1 for standard)
    ///
    /// # Returns
    /// Result containing the renderer or an error string
    pub fn from_font_bytes_with_scale(
        font_bytes: &[u8],
        font_px: f32,
        scale_factor: u32,
    ) -> Result<Self, String> {
        // Enable font hinting for sharper edges at small sizes
        let settings = fontdue::FontSettings {
            scale: font_px,
            ..fontdue::FontSettings::default()
        };
        let font =
            fontdue::Font::from_bytes(font_bytes, settings).map_err(|err| err.to_string())?;
        Ok(Self {
            font,
            font_px,
            scale_factor,
        })
    }

    /// Renders text as a tray icon image
    ///
    /// # Arguments
    /// * `text` - The text string to render
    /// * `size_px` - The desired icon size in pixels
    ///
    /// # Returns
    /// A TrayImage containing the rendered text as RGBA pixel data
    pub fn render_text_only(&self, text: &str, size_px: u32) -> TrayImage {
        // Use configured scale factor (2x for Retina, varies for Windows)
        let scaled_size = size_px * self.scale_factor;
        let scaled_font_px = self.font_px * self.scale_factor as f32;
        let padding_x = 4 * self.scale_factor as i32; // Scaled padding

        // First pass: Calculate total width at scaled size
        let mut total_width = 0;
        let mut char_metrics = Vec::with_capacity(text.len());

        for ch in text.chars() {
            let (metrics, alpha) = self.font.rasterize(ch, scaled_font_px);
            char_metrics.push((metrics, alpha));
            total_width += metrics.advance_width.round() as i32;
        }

        let content_width = total_width + (padding_x * 2);
        let width = (content_width as u32).max(scaled_size);
        let height = scaled_size;

        let mut pixmap = Pixmap::new(width, height).expect("pixmap");
        let rgba = pixmap.data_mut();

        // Clear to transparent
        for px in rgba.chunks_exact_mut(4) {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
        }

        // Better vertical centering calculation
        let baseline = (height as f32 * 0.75) as i32; // Standard typographic baseline

        // Center text horizontally
        let mut pen_x = ((width as i32 - total_width) / 2).max(0);

        for (metrics, alpha) in char_metrics {
            let glyph_w = metrics.width as i32;
            let glyph_h = metrics.height as i32;
            let glyph_x = pen_x + metrics.xmin;
            let glyph_y = baseline - metrics.ymin - glyph_h;

            for y in 0..glyph_h {
                let dst_y = glyph_y + y;
                if dst_y < 0 || dst_y >= height as i32 {
                    continue;
                }
                for x in 0..glyph_w {
                    let dst_x = glyph_x + x;
                    if dst_x < 0 || dst_x >= width as i32 {
                        continue;
                    }
                    let src_index = (y as usize * metrics.width) + x as usize;
                    let a = *alpha.get(src_index).unwrap_or(&0);

                    if a > 0 {
                        let dst_index = ((dst_y as u32 * width + dst_x as u32) * 4) as usize;
                        // Use full white with alpha for maximum contrast
                        // This makes text look white/bright instead of gray
                        rgba[dst_index] = 255; // R
                        rgba[dst_index + 1] = 255; // G
                        rgba[dst_index + 2] = 255; // B
                        rgba[dst_index + 3] = a; // Alpha
                    }
                }
            }

            pen_x += metrics.advance_width.round() as i32;
        }

        TrayImage::new(pixmap.data().to_vec(), width, height)
    }
}
