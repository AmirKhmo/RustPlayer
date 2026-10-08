//! Thin wrapper around libmpv: options, commands, property getters.
use anyhow::{anyhow, Result};
use libmpv2::Mpv;
use std::{
    ffi::CString,
    fs,
    os::raw::c_char,
    path::{Path, PathBuf},
};

pub fn config_dir() -> PathBuf {
    let d = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("rustplayer");
    let _ = fs::create_dir_all(d.join("watch_later"));
    let _ = fs::create_dir_all(d.join("screenshots"));
    d
}

pub struct Player {
    pub mpv: Mpv,
}

impl Player {
    pub fn new(cfg: &Path) -> Result<Self> {
        let watch_later = cfg.join("watch_later").to_string_lossy().into_owned();
        let shots = cfg.join("screenshots").to_string_lossy().into_owned();
        let fonts = crate::fonts::bundled_dir().to_string_lossy().into_owned();
        let mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv")?;
            // Options unknown to older mpv builds are ignored instead of failing startup.
            for (k, v) in [
                ("hwdec", "auto-safe"),
                ("idle", "yes"),
                ("keep-open", "yes"),
                ("input-default-bindings", "no"),
                ("input-vo-keyboard", "no"),
                ("osc", "no"),
                ("osd-bar", "no"),
                ("osd-level", "1"),
                ("osd-font-size", "34"),
                ("video-timing-offset", "0"),
                ("save-position-on-quit", "yes"),
                ("sub-auto", "fuzzy"),
                ("audio-file-auto", "fuzzy"),
                ("slang", "fa,per,fas,en,eng"),
                ("alang", "fa,per,fas,en,eng"),
                ("sub-codepage", "cp1256"), // UTF-8 if valid, else Persian/Arabic
                ("sub-font", "Vazirmatn"),
                ("osd-font", "Vazirmatn"),
                ("sub-font-size", "46"),
                ("sub-border-size", "2.5"),
                ("sub-shadow-offset", "1"),
                ("sub-shadow-color", "#99000000"),
                ("volume-max", "200"),
                ("autocreate-playlist", "same"),
                ("screenshot-format", "png"),
                ("ytdl", "yes"),
                ("cache", "auto"),
            ] {
                let _ = init.set_property(k, v);
            }
            let _ = init.set_property("watch-later-directory", watch_later.as_str());
            let _ = init.set_property("screenshot-directory", shots.as_str());
            let _ = init.set_property("sub-fonts-dir", fonts.as_str());
            let _ = init.set_property("osd-fonts-dir", fonts.as_str());
            Ok(())
        })
        .map_err(|e| anyhow!("mpv init failed: {e:?}"))?;
        Ok(Self { mpv })
    }

    /// Run an mpv command with raw arguments (no quoting issues with paths).
    pub fn cmd(&self, args: &[&str]) {
        let owned: Vec<CString> = args.iter().filter_map(|a| CString::new(*a).ok()).collect();
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|c| c.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        unsafe {
            libmpv2_sys::mpv_command(self.mpv.ctx.as_ptr(), ptrs.as_mut_ptr());
        }
    }

    pub fn set(&self, name: &str, value: &str) {
        self.cmd(&["set", name, value]);
    }
    pub fn osd(&self, text: &str) {
        self.cmd(&["show-text", text, "1500"]);
    }
    pub fn f64(&self, name: &str) -> Option<f64> {
        self.mpv.get_property::<f64>(name).ok()
    }
    pub fn i64(&self, name: &str) -> Option<i64> {
        self.mpv.get_property::<i64>(name).ok()
    }
    pub fn bool(&self, name: &str) -> Option<bool> {
        self.mpv.get_property::<bool>(name).ok()
    }
    pub fn str(&self, name: &str) -> Option<String> {
        self.mpv.get_property::<String>(name).ok().filter(|s| !s.is_empty())
    }
}
