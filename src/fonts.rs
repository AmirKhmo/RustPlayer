//! Loads a font that has Persian presentation-form glyphs (needed by fa.rs).
use ab_glyph::Font;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::{fs, path::PathBuf};

/// Folder with bundled fonts (next to the exe, or the repo's assets in dev builds).
pub fn bundled_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            let p = d.join("fonts");
            if p.exists() {
                return p;
            }
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts"))
}

fn candidates() -> Vec<PathBuf> {
    let mut v = vec![
        bundled_dir().join("Vazirmatn-Regular.ttf"),
    ];
    #[cfg(windows)]
    {
        let win = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        for f in ["tahoma.ttf", "segoeui.ttf", "arial.ttf"] {
            v.push(PathBuf::from(&win).join("Fonts").join(f));
        }
    }
    #[cfg(not(windows))]
    for f in [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
    ] {
        v.push(PathBuf::from(f));
    }
    v
}

/// Returns the font file that was installed, if any.
pub fn install(ctx: &egui::Context) -> Option<PathBuf> {
    // beh-initial, gaf-medial, farsi-yeh-initial, lam-alef
    const PROBE: [char; 4] = ['\u{FE91}', '\u{FB94}', '\u{FBFE}', '\u{FEFB}'];
    for path in candidates() {
        let Ok(bytes) = fs::read(&path) else { continue };
        let ok = ab_glyph::FontRef::try_from_slice(&bytes)
            .map(|f| PROBE.iter().all(|&c| f.glyph_id(c).0 != 0))
            .unwrap_or(false);
        if !ok {
            continue;
        }
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("fa".into(), FontData::from_owned(bytes));
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "fa".into());
        fonts.families.entry(FontFamily::Monospace).or_default().push("fa".into());
        ctx.set_fonts(fonts);
        return Some(path);
    }
    None
}
