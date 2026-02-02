use tray_icon_renderer::{DigitAtlas, GlyphBitmap, TrayIconRenderer};

fn make_glyph(width: usize, height: usize, alpha: Vec<u8>) -> GlyphBitmap {
    GlyphBitmap::new(width, height, 0, 0, width as f32, alpha)
}

#[test]
fn render_text_produces_non_empty_pixels() {
    let blank = make_glyph(1, 1, vec![0]);
    let one = make_glyph(2, 2, vec![255, 255, 255, 255]);

    let glyphs = core::array::from_fn(|i| if i == 1 { one.clone() } else { blank.clone() });
    let atlas = DigitAtlas::from_glyphs(glyphs, 10.0);
    let renderer = TrayIconRenderer::new(atlas);

    let image = renderer.render_text("11", 4);
    assert_eq!(image.width(), 4);
    assert_eq!(image.height(), 4);
    assert!(image.rgba().iter().any(|b| *b != 0));
}

#[test]
fn render_text_ignores_non_digits() {
    let blank = make_glyph(1, 1, vec![0]);
    let glyphs = core::array::from_fn(|_| blank.clone());
    let atlas = DigitAtlas::from_glyphs(glyphs, 10.0);
    let renderer = TrayIconRenderer::new(atlas);

    let image = renderer.render_text("ab", 4);
    assert!(image.rgba().iter().all(|b| *b == 0));
}

#[test]
fn render_text_from_font_bytes() {
    let font_bytes = include_bytes!("../assets/fonts/Arimo[wght].ttf");
    let renderer = TrayIconRenderer::from_font_bytes(font_bytes, 12.0)
        .expect("renderer from font");

    let image = renderer.render_text("12", 16);
    assert_eq!(image.width(), 16);
    assert_eq!(image.height(), 16);
    assert!(image.rgba().iter().any(|b| *b != 0));
}
