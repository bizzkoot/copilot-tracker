//! Platform-specific system text color detection for tray icon rendering.
//! Returns (r, g, b) tuple for best-contrast text color.

#![allow(unexpected_cfgs)]

pub type RgbColor = (u8, u8, u8);

pub fn text_color_for_theme_preference(theme: &str) -> RgbColor {
    match theme.to_ascii_lowercase().as_str() {
        "dark" => (255, 255, 255),
        "light" => (0, 0, 0),
        _ => detect_system_text_color(),
    }
}

#[cfg(target_os = "macos")]
pub fn detect_system_text_color() -> RgbColor {
    // Prefer the global macOS appearance setting for menu bar parity.
    // `defaults read -g AppleInterfaceStyle` returns "Dark" in dark mode and exits non-zero in light mode.
    if let Ok(output) = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        if output.status.success() {
            let style = String::from_utf8_lossy(&output.stdout);
            if style.to_ascii_lowercase().contains("dark") {
                return (255, 255, 255);
            }
            return (0, 0, 0);
        }
    }

    // Fallback: query NSApp.effectiveAppearance or NSAppearance.currentDrawingAppearance
    use cocoa::appkit::NSApp;
    use cocoa::base::nil;
    use objc::{msg_send, sel, sel_impl};
    use objc::runtime::{Class, Object};

    unsafe {
        let ns_app = NSApp();
        let appearance: *mut Object = if ns_app != nil {
            let appearance: *mut Object = msg_send![ns_app, effectiveAppearance];
            appearance
        } else {
            // Fallback: use currentDrawingAppearance
            let nsappearance_class = Class::get("NSAppearance").unwrap();
            let appearance: *mut Object = msg_send![nsappearance_class, currentDrawingAppearance];
            appearance
        };

        if !appearance.is_null() {
            let name: *mut Object = msg_send![appearance, name];
            if !name.is_null() {
                let utf8: *const std::os::raw::c_char = msg_send![name, UTF8String];
                if !utf8.is_null() {
                    let cstr = std::ffi::CStr::from_ptr(utf8);
                    let name_str = cstr.to_string_lossy();
                    if name_str.contains("Dark") {
                        return (255, 255, 255);
                    }
                }
            }
        }
    }

    (0, 0, 0)
}

#[cfg(target_os = "windows")]
pub fn detect_system_text_color() -> RgbColor {
    // On Windows, prefer system UI theme (taskbar/tray), then fallback to app theme.
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(personalize) =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
    {
        if let Ok(system_light_theme) = personalize.get_value::<u32, _>("SystemUsesLightTheme") {
            if system_light_theme == 0 {
                return (255, 255, 255);
            }
            return (0, 0, 0);
        }

        if let Ok(light_theme) = personalize.get_value::<u32, _>("AppsUseLightTheme") {
            if light_theme == 0 {
                return (255, 255, 255);
            }
            return (0, 0, 0);
        }
    }

    // Conservative fallback: assume light tray background.
    (0, 0, 0)
}

#[cfg(target_os = "linux")]
pub fn detect_system_text_color() -> RgbColor {
    // Linux desktop environments are fragmented; use a robust best-effort chain.
    if let Ok(theme) = std::env::var("GTK_THEME") {
        if theme.to_ascii_lowercase().contains("dark") {
            return (255, 255, 255);
        }
    }

    if let Ok(theme) = std::env::var("KDE_COLOR_SCHEME") {
        if theme.to_ascii_lowercase().contains("dark") {
            return (255, 255, 255);
        }
    }

    if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
        if let Some(bg) = colorfgbg.split(';').next_back().and_then(|v| v.parse::<u8>().ok()) {
            if bg <= 6 {
                return (255, 255, 255);
            }
            return (0, 0, 0);
        }
    }

    // Safe fallback for most modern Linux trays (often dark).
    (255, 255, 255)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn detect_system_text_color() -> RgbColor {
    (0, 0, 0)
}
