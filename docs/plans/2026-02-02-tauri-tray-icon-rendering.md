# Tauri Tray Icon Rendering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a dynamic tray icon renderer in a Tauri v2 Rust main process that draws tiny numeric counters and optional progress indicators using tiny-skia and fontdue.

**Architecture:** A small renderer module caches digit glyphs, blits them into a tiny RGBA buffer, and updates the tray icon via `TrayIcon::set_icon`. tiny-skia is used for compositing, while fontdue provides glyph rasterization.

**Tech Stack:** Tauri v2 (Rust), tiny-skia, fontdue

---

### Task 1: Ensure Tauri v2 scaffold exists

**Files:**
- Create (if missing): src-tauri/
- Modify (if missing): src-tauri/src/main.rs

**Step 1: Verify scaffold exists**
- Check for src-tauri/src/main.rs.
- If missing, create a Tauri v2 scaffold per migration docs.

**Step 2: Run a baseline check**
Run: cargo check
Expected: success (or fix build issues before proceeding)

**Step 3: Commit (if scaffold created)**
Run:
```bash
git add src-tauri
git commit -m "chore: add tauri v2 scaffold"
```

### Task 2: Add rendering dependencies and font asset

**Files:**
- Modify: src-tauri/Cargo.toml
- Create: src-tauri/assets/fonts/Inter-Regular.ttf (or chosen font)

**Step 1: Add dependencies**
Add to Cargo.toml:
```toml
[dependencies]
fontdue = "0.9"
tiny-skia = "0.11"
```

**Step 2: Add font asset**
- Add a font file to src-tauri/assets/fonts/.
- Decide whether to load via include_bytes! or from filesystem at runtime.

**Step 3: Commit**
```bash
git add src-tauri/Cargo.toml src-tauri/assets/fonts
 git commit -m "chore: add tray icon rendering dependencies"
```

### Task 3: Implement DigitAtlas and renderer module

**Files:**
- Create: src-tauri/src/tray_icon_renderer.rs

**Step 1: Write a failing test**
```rust
#[test]
fn render_text_produces_non_empty_pixels() {
    let font_bytes = include_bytes!("../assets/fonts/Inter-Regular.ttf");
    let font = fontdue::Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default()).unwrap();
    let renderer = TrayIconRenderer::new(font, 20.0);
    let image = renderer.render_text("123", 20);
    let rgba = image.rgba();
    assert!(rgba.iter().any(|row| row.iter().any(|b| *b != 0)));
}
```

**Step 2: Run test to verify it fails**
Run: cargo test tray_icon_renderer
Expected: FAIL (TrayIconRenderer not defined)

**Step 3: Implement minimal renderer**
```rust
pub struct TrayIconRenderer {
    font: fontdue::Font,
    atlas: DigitAtlas,
}

impl TrayIconRenderer {
    pub fn new(font: fontdue::Font, font_px: f32) -> Self {
        let atlas = DigitAtlas::new(&font, font_px);
        Self { font, atlas }
    }

    pub fn render_text(&self, text: &str, size_px: u32) -> tauri::image::Image<'static> {
        // Allocate pixmap, clear, blit digit bitmaps, return Image::new_owned
        unimplemented!()
    }
}
```

**Step 4: Run test to verify it passes**
Run: cargo test tray_icon_renderer
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/tray_icon_renderer.rs
 git commit -m "feat: add tray icon text renderer"
```

### Task 4: Wire renderer into tray update path

**Files:**
- Modify: src-tauri/src/main.rs

**Step 1: Add renderer initialization**
- Load the font at startup.
- Create a TrayIconRenderer instance.

**Step 2: Update tray icon when usage changes**
- Format the numeric string.
- Call renderer.render_text and tray.set_icon(Some(image)).
- On macOS call tray.set_icon_as_template(true).

**Step 3: Add a debounce for updates**
- Limit updates to at most 1 per second, especially on Linux.

**Step 4: Commit**
```bash
git add src-tauri/src/main.rs
 git commit -m "feat: update tray icon dynamically"
```

### Task 5: Manual verification

**Files:**
- Modify: docs/PROGRESS.md (optional)

**Step 1: Run the app and observe tray icon**
- Check that numbers render clearly at 16-20 px.
- Verify macOS template behavior and Linux update stability.

**Step 2: Commit**
```bash
git add docs/PROGRESS.md
 git commit -m "docs: note tray icon rendering verification"
```

---

## Execution Handoff

Plan complete and saved to docs/plans/2026-02-02-tauri-tray-icon-rendering.md. Two execution options:

1. Subagent-Driven (this session) - dispatch fresh subagent per task with review checkpoints
2. Parallel Session (separate) - open a new session using superpowers:executing-plans

Which approach should I take?
