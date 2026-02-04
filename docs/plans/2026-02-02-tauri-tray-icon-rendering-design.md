# Tauri Tray Icon Rendering Design

Date: 2026-02-02

## Summary

This design describes a dynamic tray icon renderer for a future Tauri v2 Rust main process. The goal is to render small numeric usage counters and optional progress indicators into a tiny RGBA bitmap, then push updates to the tray icon at runtime. The renderer uses tiny-skia for compositing and fontdue for glyph rasterization, since tiny-skia does not support text rendering.

## Goals

- Render small numeric text for tray icons with low latency and minimal dependencies.
- Support dynamic updates via Tauri tray icon APIs.
- Keep the rendering pipeline CPU-only, small, and predictable.
- Respect cross-platform tray constraints (Linux temp icon files, macOS template icons).

## Constraints

- tiny-skia does not provide text rendering; glyph rasterization must be external.
- Tray icons are tiny (16-20 px), so layout must be simple and high-contrast.
- On Linux, tray icon updates may write temporary files; update cadence should be limited.
- On macOS, template icons should be monochrome for proper light/dark behavior.

## Architecture

- TrayIconRenderer: owns a loaded Font, a digit atlas, and exposes render methods.
- DigitAtlas: pre-rasterizes digits 0-9 at the target size and caches metrics.
- Renderer outputs an RGBA buffer and constructs a tauri::image::Image.
- Tray update code calls tray.set_icon(Some(image)) when usage changes.

## Data Flow

1. Usage data arrives (used, limit, add-on cost).
2. Text is formatted (numeric only, short width).
3. Digit atlas bitmaps are blitted into a tiny-skia pixmap.
4. Optional progress ring is drawn using tiny-skia stroking.
5. The RGBA buffer is wrapped in a Tauri Image and applied to the tray icon.

## Decisions

- Use fontdue for fast rasterization and simple API.
- Use tiny-skia for compositing only.
- Cache digits instead of rasterizing text on every update.

## Risks and Mitigations

- Font rendering alignment issues at tiny sizes: test with multiple font sizes.
- Excessive updates on Linux: debounce or throttle to 1 update per second or less.

## Open Questions

- Which font file should be bundled for consistent cross-platform output?
- Should we also include a fallback static icon for error cases?
