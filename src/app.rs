//! egui interface in Persian with right-to-left layout.
//! All on-screen strings go through `t()` (fa.rs) for shaping; mpv OSD gets raw text (libass shapes it).
use crate::{
    fa::{self, t},
    mpv::{config_dir, Player},
    subs::{self, Cue},
    video::{Shared, VideoRenderer},
};
use eframe::{
    egui::{
        self, Align, Align2, Color32, FontId, Key, Layout, Modifiers, Response, RichText, Sense, Stroke,
        Ui, Vec2,
    },
    egui_glow, glow,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const ACCENT: Color32 = Color32::from_rgb(0xE9, 0xA8, 0x00);
const EMBER: Color32 = Color32::from_rgb(0xBF, 0x54, 0x2E);
const WINE: Color32 = Color32::from_rgb(0x5B, 0x21, 0x25);
const PANEL: Color32 = Color32::from_rgb(0x1C, 0x18, 0x17);
const WINDOW: Color32 = Color32::from_rgb(0x24, 0x1D, 0x1D);
const DEEP: Color32 = Color32::from_rgb(0x12, 0x0F, 0x0E);
const MUTED: Color32 = Color32::from_rgb(0x9C, 0x93, 0x88);
const TEXT: Color32 = Color32::from_rgb(0xE8, 0xE2, 0xDA);

const EQ_FREQS: [u32; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
const EQ_LABELS: [&str; 10] = ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];
const EQ_PRESETS: [(&str, [f32; 10]); 5] = [
    ("تخت", [0.0; 10]),
    ("بیس", [6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ("صدای خواننده", [-2.0, -2.0, 0.0, 2.0, 4.0, 4.0, 3.0, 1.0, 0.0, -1.0]),
    ("زیر", [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 5.0, 6.0, 6.0]),
    ("راک", [4.0, 3.0, 1.0, -1.0, -2.0, -1.0, 1.0, 3.0, 4.0, 4.0]),
];
const COLOR_PROPS: [(&str, &str); 5] = [
    ("brightness", "روشنایی"),
    ("contrast", "کنتراست"),
    ("saturation", "اشباع رنگ"),
    ("gamma", "گاما"),
    ("hue", "ته‌رنگ"),
];
const ASPECTS: [(&str, &str); 6] = [
    ("پیش‌فرض", "no"),
    ("16:9", "16:9"),
    ("4:3", "4:3"),
    ("21:9", "2.33"),
    ("2.35:1", "2.35"),
    ("1:1", "1"),
];
const SUB_FONTS: [&str; 7] = ["Vazirmatn", "Tahoma", "B Nazanin", "B Yekan", "IRANSans", "Segoe UI", "Arial"];
const MEDIA_EXT: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "webm", "flv", "wmv", "ts", "m2ts", "mts", "mpg", "mpeg", "m4v",
    "3gp", "ogv", "vob", "mp3", "flac", "wav", "ogg", "m4a", "opus", "aac", "wma", "m3u", "m3u8",
];
const SUB_EXT: &[&str] = &["srt", "ass", "ssa", "vtt", "sub", "idx", "sup", "smi"];
const CUE_H: f32 = 62.0;

#[derive(Clone, Default)]
struct Track {
    id: i64,
    kind: String,
    label: String,
    selected: bool,
    external: Option<String>,
}

#[derive(Default)]
struct Status {
    idle: bool,
    paused: bool,
    pos: f64,
    dur: f64,
    vol: f64,
    mute: bool,
    speed: f64,
    ab_a: Option<f64>,
    ab_b: Option<f64>,
    sub_delay: f64,
    audio_delay: f64,
    sub_text: String,
    title: String,
    playlist: Vec<(String, bool)>,
    tracks: Vec<Track>,
    secondary_sid: String,
    chapters: Vec<(String, f64)>,
    loop_file: bool,
    loop_playlist: bool,
    sub_visible: bool,
}

struct Prefs {
    color: [i32; 5],
    zoom: f64,
    rotate: i64,
    aspect: String,
    scale: String,
    deint: bool,
    hwdec: bool,
    eq: [f32; 10],
    normalize: bool,
    channels: String,
    sub_scale: f64,
    sub_pos: i64,
    sub_border: f64,
    sub_size: i64,
    sub_bold: bool,
    sub_box: bool,
    sub_font: String,
    sub_color: Color32,
    codepage: String,
    resume: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            color: [0; 5],
            zoom: 0.0,
            rotate: 0,
            aspect: "no".into(),
            scale: "spline36".into(),
            deint: false,
            hwdec: true,
            eq: [0.0; 10],
            normalize: false,
            channels: "auto-safe".into(),
            sub_scale: 1.0,
            sub_pos: 100,
            sub_border: 2.5,
            sub_size: 46,
            sub_bold: false,
            sub_box: false,
            sub_font: "Vazirmatn".into(),
            sub_color: Color32::WHITE,
            codepage: "cp1256".into(),
            resume: true,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Video,
    Audio,
    Subtitles,
    Playback,
}

pub struct App {
    player: Player,
    video: Arc<Shared>,
    cfg: PathBuf,
    st: Status,
    pf: Prefs,
    recent: Vec<String>,
    show_playlist: bool,
    show_subs: bool,
    show_info: bool,
    show_prefs: bool,
    show_url: bool,
    show_keys: bool,
    tab: Tab,
    url: String,
    ontop: bool,
    last_move: Instant,
    last_slow: Instant,
    seek_drag: Option<f64>,
    scroll_acc: f32,
    af_dirty: bool,
    win_title: String,
    // subtitle explorer
    sub_key: Option<String>,
    cues: Vec<Cue>,
    live: Vec<Cue>,
    sub_search: String,
    follow: bool,
    last_cue: Option<usize>,
    no_font_warning: bool,
}

// ---------- small helpers ----------
fn fmt_time(t: f64) -> String {
    let t = t.max(0.0) as u64;
    let (h, m, s) = (t / 3600, (t / 60) % 60, t % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn file_name(p: &str) -> String {
    Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string())
}

fn has_ext(p: &str, list: &[&str]) -> bool {
    Path::new(p)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| list.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// A single right-to-left row (keeps the row height small).
fn rtl<R>(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| ui.with_layout(Layout::right_to_left(Align::Center), f).inner).inner
}

/// Label on the right, control(s) to its left.
fn field(ui: &mut Ui, label: &str, f: impl FnOnce(&mut Ui)) {
    rtl(ui, |ui| {
        ui.allocate_ui_with_layout(Vec2::new(120.0, 22.0), Layout::right_to_left(Align::Center), |ui| {
            ui.set_min_width(120.0);
            ui.label(t(label));
        });
        f(ui);
    });
}

/// Right-aligned vertical stack (menus, lists).
fn stack<R>(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> R {
    ui.with_layout(Layout::top_down_justified(Align::Max), f).inner
}

/// Menu item: Persian text on the right, shortcut on the left.
fn mi(ui: &mut Ui, text: &str, shortcut: &str) -> Response {
    if shortcut.is_empty() {
        ui.button(t(text))
    } else {
        ui.button(format!("{shortcut}     {}", t(text)))
    }
}

/// Menu item with a check mark on the right.
fn mc(ui: &mut Ui, checked: bool, text: &str, shortcut: &str) -> Response {
    let mark = if checked { "✔" } else { "   " };
    let sc = if shortcut.is_empty() { String::new() } else { format!("{shortcut}     ") };
    ui.button(format!("{sc}{}  {mark}", t(text)))
}

fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = WINDOW;
    v.extreme_bg_color = DEEP;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0_f32, Color32::BLACK);
    v.hyperlink_color = ACCENT;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    v.window_rounding = 8.0.into();
    style.visuals = v;
    use egui::TextStyle::*;
    style.text_styles.insert(Body, FontId::proportional(15.0));
    style.text_styles.insert(Button, FontId::proportional(15.0));
    style.text_styles.insert(Heading, FontId::proportional(20.0));
    style.text_styles.insert(Monospace, FontId::monospace(14.0));
    style.text_styles.insert(Small, FontId::proportional(12.0));
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    ctx.set_style(style);
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<String>) -> Self {
        apply_style(&cc.egui_ctx);
        let font = crate::fonts::install(&cc.egui_ctx);
        let cfg = config_dir();
        let player = Player::new(&cfg).expect("could not start libmpv (is libmpv-2.dll next to the exe?)");
        let loader = cc.get_proc_address.expect("RustPlayer needs the OpenGL (glow) backend");
        let renderer = VideoRenderer::new(&player, loader, cc.egui_ctx.clone())
            .expect("could not create the mpv OpenGL renderer");
        let recent = fs::read_to_string(cfg.join("recent.txt"))
            .map(|s| s.lines().filter(|l| !l.is_empty()).map(String::from).collect())
            .unwrap_or_default();
        let mut app = Self {
            player,
            video: Arc::new(Shared(Mutex::new(Some(renderer)))),
            cfg,
            st: Status { idle: true, vol: 100.0, speed: 1.0, sub_visible: true, ..Default::default() },
            pf: Prefs::default(),
            recent,
            show_playlist: false,
            show_subs: false,
            show_info: false,
            show_prefs: false,
            show_url: false,
            show_keys: false,
            tab: Tab::Video,
            url: String::new(),
            ontop: false,
            last_move: Instant::now(),
            last_slow: Instant::now() - Duration::from_secs(5),
            seek_drag: None,
            scroll_acc: 0.0,
            af_dirty: false,
            win_title: String::new(),
            sub_key: None,
            cues: Vec::new(),
            live: Vec::new(),
            sub_search: String::new(),
            follow: true,
            last_cue: None,
            no_font_warning: font.is_none(),
        };
        app.player.set("scale", "spline36");
        if !files.is_empty() {
            app.open_paths(files, false);
        }
        app
    }

    // ---------- mpv helpers ----------
    fn cmd(&self, a: &[&str]) {
        self.player.cmd(a)
    }
    fn set(&self, k: &str, v: &str) {
        self.player.set(k, v)
    }
    /// OSD text goes to mpv/libass in logical order (libass shapes Persian itself).
    fn osd(&self, msg: impl AsRef<str>) {
        self.player.osd(msg.as_ref())
    }
    fn refresh_soon(&mut self) {
        self.last_slow = Instant::now() - Duration::from_secs(5);
    }

    fn poll(&mut self) {
        let p = &self.player;
        let s = &mut self.st;
        s.idle = p.bool("idle-active").unwrap_or(true);
        s.paused = p.bool("pause").unwrap_or(false);
        s.pos = p.f64("time-pos").unwrap_or(0.0);
        s.dur = p.f64("duration").unwrap_or(0.0);
        s.vol = p.f64("volume").unwrap_or(100.0);
        s.mute = p.bool("mute").unwrap_or(false);
        s.speed = p.f64("speed").unwrap_or(1.0);
        s.ab_a = p.f64("ab-loop-a");
        s.ab_b = p.f64("ab-loop-b");
        s.sub_delay = p.f64("sub-delay").unwrap_or(0.0);
        s.audio_delay = p.f64("audio-delay").unwrap_or(0.0);

        // live subtitle line (works for embedded tracks too)
        let txt = p.str("sub-text").unwrap_or_default();
        if !txt.is_empty() && txt != s.sub_text && self.cues.is_empty() {
            let start = p.f64("sub-start").unwrap_or(s.pos);
            let end = p.f64("sub-end").unwrap_or(start + 2.0);
            if self.live.last().map(|c| c.text != txt).unwrap_or(true) {
                self.live.push(Cue::new(start, end, txt.clone()));
                if self.live.len() > 3000 {
                    self.live.remove(0);
                }
            }
        }
        s.sub_text = txt;

        if self.last_slow.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_slow = Instant::now();
        s.title = p.str("media-title").unwrap_or_default();
        s.loop_file = p.str("loop-file").map(|v| v != "no").unwrap_or(false);
        s.loop_playlist = p.str("loop-playlist").map(|v| v != "no").unwrap_or(false);
        s.sub_visible = p.bool("sub-visibility").unwrap_or(true);
        s.secondary_sid = p.str("secondary-sid").unwrap_or_else(|| "no".into());

        let n = p.i64("playlist/count").unwrap_or(0);
        s.playlist = (0..n)
            .map(|i| {
                let f = p.str(&format!("playlist/{i}/filename")).unwrap_or_default();
                let name = p.str(&format!("playlist/{i}/title")).unwrap_or_else(|| file_name(&f));
                (name, p.bool(&format!("playlist/{i}/current")).unwrap_or(false))
            })
            .collect();

        let n = p.i64("track-list/count").unwrap_or(0);
        s.tracks = (0..n)
            .map(|i| {
                let g = |k: &str| p.str(&format!("track-list/{i}/{k}"));
                let id = p.i64(&format!("track-list/{i}/id")).unwrap_or(0);
                let external = if p.bool(&format!("track-list/{i}/external")).unwrap_or(false) {
                    g("external-filename")
                } else {
                    None
                };
                let mut label = format!("#{id}");
                if let Some(t) = g("title") {
                    label += &format!(" {t}");
                }
                if let Some(l) = g("lang") {
                    label += &format!(" [{l}]");
                }
                if let Some(c) = g("codec") {
                    label += &format!(" ({c})");
                }
                if external.is_some() {
                    label += " - فایل خارجی";
                }
                Track {
                    id,
                    kind: g("type").unwrap_or_default(),
                    label,
                    selected: p.bool(&format!("track-list/{i}/selected")).unwrap_or(false),
                    external,
                }
            })
            .collect();

        let n = p.i64("chapter-list/count").unwrap_or(0);
        s.chapters = (0..n)
            .map(|i| {
                let t = p.f64(&format!("chapter-list/{i}/time")).unwrap_or(0.0);
                let name = p
                    .str(&format!("chapter-list/{i}/title"))
                    .unwrap_or_else(|| format!("فصل {}", i + 1));
                (name, t)
            })
            .collect();

        self.sync_sub_source();
    }

    /// Reload the explorer list whenever the selected subtitle track changes.
    fn sync_sub_source(&mut self) {
        let sel = self.st.tracks.iter().find(|t| t.kind == "sub" && t.selected).cloned();
        let key = sel
            .as_ref()
            .map(|t| t.external.clone().unwrap_or_else(|| format!("embedded:{}", t.id)));
        if key != self.sub_key {
            self.sub_key = key;
            self.cues = sel.and_then(|t| t.external).map(|p| subs::load(&p)).unwrap_or_default();
            self.live.clear();
            self.last_cue = None;
        }
    }

    // ---------- actions ----------
    fn toggle_pause(&self) {
        self.cmd(&["cycle", "pause"]);
    }
    fn seek(&self, secs: f64) {
        self.cmd(&["seek", &secs.to_string(), "relative"]);
    }
    fn add_volume(&self, d: f64) {
        let v = (self.st.vol + d).clamp(0.0, 200.0);
        self.set("volume", &format!("{v:.0}"));
        self.osd(format!("صدا {v:.0}%"));
    }
    fn set_speed(&self, s: f64) {
        let s = s.clamp(0.1, 8.0);
        self.set("speed", &format!("{s:.2}"));
        self.osd(format!("سرعت {s:.2}x"));
    }
    fn stop(&mut self) {
        if !self.st.idle {
            self.cmd(&["write-watch-later-config"]);
        }
        self.cmd(&["stop"]);
        self.refresh_soon();
    }
    fn screenshot(&self, with_subs: bool) {
        self.cmd(&["screenshot", if with_subs { "subtitles" } else { "video" }]);
    }
    fn adj_color(&mut self, i: usize, d: i32) {
        let c = &mut self.pf.color[i];
        *c = (*c + d).clamp(-100, 100);
        let v = *c;
        self.set(COLOR_PROPS[i].0, &v.to_string());
        self.osd(format!("{} {v}", COLOR_PROPS[i].1));
    }
    fn reset_color(&mut self) {
        self.pf.color = [0; 5];
        for (p, _) in COLOR_PROPS {
            self.set(p, "0");
        }
        self.osd("تنظیمات تصویر بازنشانی شد");
    }
    fn set_aspect(&mut self, v: &str) {
        self.pf.aspect = v.to_string();
        self.set("video-aspect-override", v);
        let label = ASPECTS.iter().find(|a| a.1 == v).map(|a| a.0).unwrap_or(v);
        self.osd(format!("نسبت تصویر: {label}"));
    }
    fn cycle_aspect(&mut self) {
        let i = ASPECTS.iter().position(|a| a.1 == self.pf.aspect).unwrap_or(0);
        self.set_aspect(ASPECTS[(i + 1) % ASPECTS.len()].1);
    }
    fn rotate(&mut self) {
        self.pf.rotate = (self.pf.rotate + 90) % 360;
        self.set("video-rotate", &self.pf.rotate.to_string());
        self.osd(format!("چرخش {}°", self.pf.rotate));
    }
    fn set_zoom(&mut self, z: f64) {
        self.pf.zoom = z.clamp(-2.0, 2.0);
        self.set("video-zoom", &format!("{:.2}", self.pf.zoom));
    }
    fn toggle_deint(&mut self) {
        self.pf.deint = !self.pf.deint;
        self.set("deinterlace", if self.pf.deint { "yes" } else { "no" });
        self.osd(if self.pf.deint { "حذف درهم‌رفتگی: روشن" } else { "حذف درهم‌رفتگی: خاموش" });
    }
    fn toggle_hw(&mut self) {
        self.pf.hwdec = !self.pf.hwdec;
        self.set("hwdec", if self.pf.hwdec { "auto-safe" } else { "no" });
        self.osd(if self.pf.hwdec { "رمزگشایی سخت‌افزاری" } else { "رمزگشایی نرم‌افزاری" });
    }
    fn toggle_fs(&self, ctx: &egui::Context) {
        let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fs));
    }
    fn toggle_ontop(&mut self, ctx: &egui::Context) {
        self.ontop = !self.ontop;
        let lvl = if self.ontop { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(lvl));
        self.osd(if self.ontop { "همیشه رو: روشن" } else { "همیشه رو: خاموش" });
    }
    fn apply_af(&mut self) {
        self.af_dirty = false;
        let mut f: Vec<String> = Vec::new();
        if self.pf.eq.iter().any(|g| g.abs() > 0.05) {
            for (freq, g) in EQ_FREQS.iter().zip(self.pf.eq.iter()) {
                f.push(format!("equalizer=f={freq}:t=o:w=1:g={g:.1}"));
            }
        }
        if self.pf.normalize {
            f.push("dynaudnorm".into());
        }
        let chain = if f.is_empty() { String::new() } else { format!("lavfi=[{}]", f.join(",")) };
        self.set("af", &chain);
    }
    fn apply_sub_style(&self) {
        let c = self.pf.sub_color;
        self.set("sub-color", &format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()));
        self.set("sub-font", &self.pf.sub_font);
        self.set("sub-font-size", &self.pf.sub_size.to_string());
        self.set("sub-bold", if self.pf.sub_bold { "yes" } else { "no" });
        self.set("sub-border-size", &format!("{:.1}", self.pf.sub_border));
        self.set("sub-border-style", if self.pf.sub_box { "opaque-box" } else { "outline-and-shadow" });
        self.set("sub-back-color", if self.pf.sub_box { "#B0000000" } else { "#00000000" });
    }

    fn add_recent(&mut self, p: &str) {
        self.recent.retain(|r| r != p);
        self.recent.insert(0, p.to_string());
        self.recent.truncate(15);
        let _ = fs::write(self.cfg.join("recent.txt"), self.recent.join("\n"));
    }

    fn open_paths(&mut self, paths: Vec<String>, append: bool) {
        let mut first = !append;
        for p in paths {
            if has_ext(&p, SUB_EXT) && !self.st.idle {
                self.cmd(&["sub-add", &p, "select"]);
                self.osd(format!("زیرنویس: {}", file_name(&p)));
                continue;
            }
            if first {
                if !self.st.idle {
                    self.cmd(&["write-watch-later-config"]);
                }
                self.cmd(&["loadfile", &p, "replace"]);
                self.set("pause", "no");
                first = false;
            } else {
                self.cmd(&["loadfile", &p, "append-play"]);
            }
            self.add_recent(&p);
        }
        self.refresh_soon();
    }

    fn open_dialog(&mut self, append: bool) {
        let picked = rfd::FileDialog::new()
            .set_title("باز کردن فایل")
            .add_filter("Media", MEDIA_EXT)
            .add_filter("Subtitles", SUB_EXT)
            .add_filter("All files", &["*"])
            .pick_files();
        if let Some(files) = picked {
            let v = files.into_iter().map(|p| p.to_string_lossy().into_owned()).collect();
            self.open_paths(v, append);
        }
    }
    fn open_folder(&mut self) {
        if let Some(d) = rfd::FileDialog::new().set_title("باز کردن پوشه").pick_folder() {
            self.open_paths(vec![d.to_string_lossy().into_owned()], false);
        }
    }
    fn load_sub_dialog(&mut self) {
        if let Some(f) = rfd::FileDialog::new().add_filter("Subtitles", SUB_EXT).pick_file() {
            self.cmd(&["sub-add", &f.to_string_lossy(), "select"]);
            self.refresh_soon();
        }
    }

    // ---------- keyboard (PotPlayer layout) ----------
    fn keys(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let k = |m: Modifiers, key: Key| ctx.input_mut(|i| i.consume_key(m, key));
        let (c, a, n) = (Modifiers::COMMAND, Modifiers::ALT, Modifiers::NONE);

        if k(c, Key::O) { self.open_dialog(false); }
        if k(c, Key::U) { self.show_url = true; }
        if k(c, Key::E) { self.screenshot(true); }
        if k(c, Key::A) { self.cmd(&["cycle", "audio"]); }
        if k(c, Key::D) { self.toggle_deint(); }
        if k(c, Key::H) { self.toggle_hw(); }
        if k(c, Key::T) { self.toggle_ontop(ctx); }
        if k(c, Key::R) { self.rotate(); }
        if k(c, Key::ArrowLeft) { self.seek(-30.0); }
        if k(c, Key::ArrowRight) { self.seek(30.0); }
        if k(c, Key::Minus) { self.cmd(&["add", "audio-delay", "-0.05"]); }
        if k(c, Key::Equals) || k(c, Key::Plus) { self.cmd(&["add", "audio-delay", "0.05"]); }

        if k(a, Key::ArrowLeft) { self.seek(-60.0); }
        if k(a, Key::ArrowRight) { self.seek(60.0); }
        if k(a, Key::S) { self.cmd(&["cycle", "sub"]); self.refresh_soon(); }
        if k(a, Key::H) { self.cmd(&["cycle", "sub-visibility"]); }
        if k(a, Key::ArrowUp) { self.cmd(&["add", "sub-pos", "-1"]); }
        if k(a, Key::ArrowDown) { self.cmd(&["add", "sub-pos", "1"]); }

        if k(n, Key::Space) { self.toggle_pause(); }
        if k(n, Key::Enter) { self.toggle_fs(ctx); }
        if k(n, Key::Escape) && ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
        if k(n, Key::ArrowLeft) { self.seek(-5.0); }
        if k(n, Key::ArrowRight) { self.seek(5.0); }
        if k(n, Key::ArrowUp) { self.add_volume(5.0); }
        if k(n, Key::ArrowDown) { self.add_volume(-5.0); }
        if k(n, Key::M) { self.cmd(&["cycle", "mute"]); }
        if k(n, Key::C) { self.set_speed(self.st.speed + 0.1); }
        if k(n, Key::X) { self.set_speed(self.st.speed - 0.1); }
        if k(n, Key::Z) { self.set_speed(1.0); }
        if k(n, Key::F) { self.cmd(&["frame-step"]); }
        if k(n, Key::D) { self.cmd(&["frame-back-step"]); }
        if k(n, Key::PageUp) { self.cmd(&["playlist-prev"]); }
        if k(n, Key::PageDown) { self.cmd(&["playlist-next"]); }
        if k(n, Key::Home) { self.cmd(&["seek", "0", "absolute"]); }
        if k(n, Key::Backslash) { self.cmd(&["ab-loop"]); }
        if k(n, Key::Comma) { self.cmd(&["add", "sub-delay", "-0.5"]); }
        if k(n, Key::Period) { self.cmd(&["add", "sub-delay", "0.5"]); }
        if k(n, Key::Slash) { self.set("sub-delay", "0"); self.osd("تأخیر زیرنویس: صفر"); }
        if k(n, Key::A) { self.cycle_aspect(); }
        if k(n, Key::Q) { self.adj_color(0, -1); }
        if k(n, Key::W) { self.adj_color(0, 1); }
        if k(n, Key::E) { self.adj_color(1, -1); }
        if k(n, Key::R) { self.adj_color(1, 1); }
        if k(n, Key::T) { self.adj_color(2, -1); }
        if k(n, Key::Y) { self.adj_color(2, 1); }
        if k(n, Key::U) { self.adj_color(4, -1); }
        if k(n, Key::I) { self.adj_color(4, 1); }
        if k(n, Key::Backspace) { self.reset_color(); }
        if k(n, Key::Tab) { self.show_info = !self.show_info; }
        if k(n, Key::F1) { self.show_keys = !self.show_keys; }
        if k(n, Key::F5) { self.show_prefs = !self.show_prefs; }
        if k(n, Key::F6) { self.show_playlist = !self.show_playlist; }
        if k(n, Key::F7) { self.show_subs = !self.show_subs; }
    }

    // ---------- menus (top bar + right-click) ----------
    fn track_menu(&mut self, ui: &mut Ui, kind: &str, prop: &str) {
        stack(ui, |ui| {
            let tracks: Vec<Track> = self.st.tracks.iter().filter(|t| t.kind == kind).cloned().collect();
            let current_secondary = self.st.secondary_sid.clone();
            let is_sel = |t: &Track| {
                if prop == "secondary-sid" { current_secondary == t.id.to_string() } else { t.selected }
            };
            if kind == "sub" {
                let none = !tracks.iter().any(|t| is_sel(t));
                if mc(ui, none, "خاموش", "").clicked() {
                    self.set(prop, "no");
                    self.refresh_soon();
                    ui.close_menu();
                }
            }
            if tracks.is_empty() {
                ui.label(RichText::new(t("ترکی وجود ندارد")).color(MUTED));
            }
            for tr in &tracks {
                if mc(ui, is_sel(tr), &tr.label, "").clicked() {
                    self.set(prop, &tr.id.to_string());
                    self.refresh_soon();
                    ui.close_menu();
                }
            }
        });
    }

    fn menus(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.menu_button(t("فایل"), |ui| stack(ui, |ui| {
            if mi(ui, "باز کردن فایل…", "Ctrl+O").clicked() { ui.close_menu(); self.open_dialog(false); }
            if mi(ui, "افزودن به فهرست پخش…", "").clicked() { ui.close_menu(); self.open_dialog(true); }
            if mi(ui, "باز کردن پوشه…", "").clicked() { ui.close_menu(); self.open_folder(); }
            if mi(ui, "باز کردن آدرس اینترنتی…", "Ctrl+U").clicked() { ui.close_menu(); self.show_url = true; }
            ui.menu_button(t("فایل‌های اخیر"), |ui| stack(ui, |ui| {
                if self.recent.is_empty() {
                    ui.label(RichText::new(t("هنوز چیزی نیست")).color(MUTED));
                }
                for r in self.recent.clone() {
                    if ui.button(t(&file_name(&r))).on_hover_text(&r).clicked() {
                        ui.close_menu();
                        self.open_paths(vec![r], false);
                    }
                }
                if !self.recent.is_empty() {
                    ui.separator();
                    if mi(ui, "پاک کردن فهرست", "").clicked() {
                        self.recent.clear();
                        let _ = fs::remove_file(self.cfg.join("recent.txt"));
                        ui.close_menu();
                    }
                }
            }));
            ui.separator();
            if mi(ui, "بستن فایل", "").clicked() { ui.close_menu(); self.stop(); }
            if mi(ui, "خروج", "Alt+F4").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
        }));

        ui.menu_button(t("پخش"), |ui| stack(ui, |ui| {
            let label = if self.st.paused { "پخش" } else { "مکث" };
            if mi(ui, label, "Space").clicked() { self.toggle_pause(); ui.close_menu(); }
            if mi(ui, "توقف", "").clicked() { self.stop(); ui.close_menu(); }
            if mi(ui, "قبلی", "PgUp").clicked() { self.cmd(&["playlist-prev"]); ui.close_menu(); }
            if mi(ui, "بعدی", "PgDn").clicked() { self.cmd(&["playlist-next"]); ui.close_menu(); }
            ui.separator();
            ui.menu_button(t("سرعت"), |ui| stack(ui, |ui| {
                for s in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0] {
                    if mc(ui, (self.st.speed - s).abs() < 0.01, &format!("{s}x"), "").clicked() {
                        self.set_speed(s);
                        ui.close_menu();
                    }
                }
            }));
            if mi(ui, "تکرار A-B (شروع، پایان، خاموش)", "\\").clicked() { self.cmd(&["ab-loop"]); ui.close_menu(); }
            if mc(ui, self.st.loop_file, "تکرار فایل", "").clicked() {
                self.set("loop-file", if self.st.loop_file { "no" } else { "inf" });
                self.refresh_soon();
            }
            if mc(ui, self.st.loop_playlist, "تکرار فهرست پخش", "").clicked() {
                self.set("loop-playlist", if self.st.loop_playlist { "no" } else { "inf" });
                self.refresh_soon();
            }
            if !self.st.chapters.is_empty() {
                ui.menu_button(t("فصل‌ها"), |ui| stack(ui, |ui| {
                    for (i, (name, time)) in self.st.chapters.clone().into_iter().enumerate() {
                        if ui.button(format!("{}     {}", fmt_time(time), t(&name))).clicked() {
                            self.set("chapter", &i.to_string());
                            ui.close_menu();
                        }
                    }
                }));
            }
        }));

        ui.menu_button(t("تصویر"), |ui| stack(ui, |ui| {
            ui.menu_button(t("ترک تصویر"), |ui| self.track_menu(ui, "video", "vid"));
            ui.menu_button(t("نسبت تصویر"), |ui| stack(ui, |ui| {
                for (label, v) in ASPECTS {
                    if mc(ui, self.pf.aspect == v, label, "").clicked() { self.set_aspect(v); ui.close_menu(); }
                }
            }));
            if mi(ui, "چرخش ۹۰ درجه", "Ctrl+R").clicked() { self.rotate(); }
            if mi(ui, "قرینه افقی", "").clicked() { self.cmd(&["vf", "toggle", "hflip"]); }
            if mi(ui, "وارونه عمودی", "").clicked() { self.cmd(&["vf", "toggle", "vflip"]); }
            if mi(ui, "بازنشانی زوم", "").clicked() { self.set_zoom(0.0); }
            ui.separator();
            if mc(ui, self.pf.deint, "حذف درهم‌رفتگی", "Ctrl+D").clicked() { self.toggle_deint(); }
            if mc(ui, self.pf.hwdec, "رمزگشایی سخت‌افزاری", "Ctrl+H").clicked() { self.toggle_hw(); }
            ui.separator();
            if mi(ui, "عکس از صفحه", "Ctrl+E").clicked() { self.screenshot(true); ui.close_menu(); }
            if mi(ui, "عکس بدون زیرنویس", "").clicked() { self.screenshot(false); ui.close_menu(); }
            if mi(ui, "رنگ و تصویر…", "").clicked() { self.tab = Tab::Video; self.show_prefs = true; ui.close_menu(); }
        }));

        ui.menu_button(t("صدا"), |ui| stack(ui, |ui| {
            ui.menu_button(t("ترک صدا"), |ui| self.track_menu(ui, "audio", "aid"));
            if mc(ui, self.st.mute, "بی‌صدا", "M").clicked() { self.cmd(&["cycle", "mute"]); }
            if mc(ui, self.pf.normalize, "یکسان‌سازی بلندی صدا", "").clicked() {
                self.pf.normalize = !self.pf.normalize;
                self.af_dirty = true;
            }
            if mi(ui, "اکولایزر…", "").clicked() { self.tab = Tab::Audio; self.show_prefs = true; ui.close_menu(); }
            ui.separator();
            if mi(ui, "تأخیر صدا −۵۰ میلی‌ثانیه", "Ctrl+-").clicked() { self.cmd(&["add", "audio-delay", "-0.05"]); }
            if mi(ui, "تأخیر صدا +۵۰ میلی‌ثانیه", "Ctrl+=").clicked() { self.cmd(&["add", "audio-delay", "0.05"]); }
        }));

        ui.menu_button(t("زیرنویس"), |ui| stack(ui, |ui| {
            ui.menu_button(t("زیرنویس اصلی"), |ui| self.track_menu(ui, "sub", "sid"));
            ui.menu_button(t("زیرنویس دوم (دوزبانه)"), |ui| self.track_menu(ui, "sub", "secondary-sid"));
            if mi(ui, "بارگذاری فایل زیرنویس…", "").clicked() { ui.close_menu(); self.load_sub_dialog(); }
            if mc(ui, self.st.sub_visible, "نمایش زیرنویس", "Alt+H").clicked() {
                self.cmd(&["cycle", "sub-visibility"]);
                self.refresh_soon();
            }
            if mi(ui, "زیرنویس زودتر (۰٫۵ ثانیه)", ",").clicked() { self.cmd(&["add", "sub-delay", "-0.5"]); }
            if mi(ui, "زیرنویس دیرتر (۰٫۵ ثانیه)", ".").clicked() { self.cmd(&["add", "sub-delay", "0.5"]); }
            ui.separator();
            if mc(ui, self.show_subs, "مرورگر زیرنویس", "F7").clicked() { self.show_subs = !self.show_subs; ui.close_menu(); }
            if mi(ui, "ظاهر زیرنویس…", "").clicked() { self.tab = Tab::Subtitles; self.show_prefs = true; ui.close_menu(); }
        }));

        ui.menu_button(t("نما"), |ui| stack(ui, |ui| {
            if mc(ui, self.show_playlist, "فهرست پخش", "F6").clicked() { self.show_playlist = !self.show_playlist; }
            if mc(ui, self.show_subs, "مرورگر زیرنویس", "F7").clicked() { self.show_subs = !self.show_subs; }
            if mc(ui, self.show_info, "اطلاعات فایل", "Tab").clicked() { self.show_info = !self.show_info; }
            if mi(ui, "آمار پخش", "").clicked() { self.cmd(&["script-binding", "stats/display-stats-toggle"]); ui.close_menu(); }
            if mc(ui, self.ontop, "همیشه رو", "Ctrl+T").clicked() { self.toggle_ontop(ctx); }
            if mi(ui, "تمام‌صفحه", "Enter").clicked() { self.toggle_fs(ctx); ui.close_menu(); }
            ui.separator();
            if mi(ui, "تنظیمات…", "F5").clicked() { self.show_prefs = true; ui.close_menu(); }
            if mi(ui, "میانبرهای صفحه‌کلید", "F1").clicked() { self.show_keys = true; ui.close_menu(); }
        }));
    }

    // ---------- bottom bar (transport stays left-to-right, like time) ----------
    fn seekbar(&mut self, ui: &mut Ui) {
        let dur = self.st.dur;
        let mut v = self.seek_drag.unwrap_or(self.st.pos).clamp(0.0, dur.max(0.0));
        ui.spacing_mut().slider_width = ui.available_width();
        let resp = ui.add_enabled(
            dur > 0.0,
            egui::Slider::new(&mut v, 0.0..=dur.max(1.0)).show_value(false).trailing_fill(true),
        );
        if resp.changed() {
            self.seek_drag = Some(v);
            self.cmd(&["seek", &format!("{v:.3}"), "absolute+keyframes"]);
        }
        if !resp.dragged() {
            if let Some(t) = self.seek_drag.take() {
                self.cmd(&["seek", &format!("{t:.3}"), "absolute+exact"]);
            }
        }
        if dur > 0.0 {
            let r = resp.rect;
            let x = |t: f64| r.left() + (t / dur) as f32 * r.width();
            let painter = ui.painter();
            for (_, t) in &self.st.chapters {
                painter.vline(x(*t), r.center().y - 5.0..=r.center().y + 5.0, Stroke::new(1.5_f32, MUTED));
            }
            for t in [self.st.ab_a, self.st.ab_b].into_iter().flatten() {
                painter.vline(x(t), r.y_range(), Stroke::new(2.5_f32, EMBER));
            }
            if let Some(hp) = resp.hover_pos() {
                let t = ((hp.x - r.left()) / r.width()).clamp(0.0, 1.0) as f64 * dur;
                resp.on_hover_text_at_pointer(fmt_time(t));
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        self.seekbar(ui);
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let icon = |s: &str| RichText::new(s).size(18.0);
            if ui.button(icon("⏮")).on_hover_text(t("قبلی (PgUp)")).clicked() { self.cmd(&["playlist-prev"]); }
            if ui.button(icon("⏪")).on_hover_text(t("۱۰ ثانیه عقب")).clicked() { self.seek(-10.0); }
            let play = if self.st.paused || self.st.idle { "⏵" } else { "⏸" };
            let big = egui::Button::new(RichText::new(play).size(20.0).color(Color32::BLACK))
                .fill(ACCENT)
                .min_size(Vec2::new(44.0, 30.0));
            if ui.add(big).on_hover_text(t("پخش / مکث (Space)")).clicked() {
                if self.st.idle { self.open_dialog(false); } else { self.toggle_pause(); }
            }
            if ui.button(icon("⏩")).on_hover_text(t("۱۰ ثانیه جلو")).clicked() { self.seek(10.0); }
            if ui.button(icon("⏭")).on_hover_text(t("بعدی (PgDn)")).clicked() { self.cmd(&["playlist-next"]); }
            if ui.button(icon("⏹")).on_hover_text(t("توقف")).clicked() { self.stop(); }
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!("{}  /  {}", fmt_time(self.st.pos), fmt_time(self.st.dur)))
                    .monospace()
                    .size(15.0),
            );
            if (self.st.speed - 1.0).abs() > 0.01 {
                ui.label(RichText::new(format!("{:.2}x", self.st.speed)).color(ACCENT).strong());
            }
            if self.st.ab_a.is_some() {
                ui.label(RichText::new(if self.st.ab_b.is_some() { "A-B" } else { "A-" }).color(EMBER).strong());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                if ui.selectable_label(fs, icon("⛶")).on_hover_text(t("تمام‌صفحه (Enter)")).clicked() { self.toggle_fs(ctx); }
                if ui.selectable_label(self.show_playlist, icon("☰")).on_hover_text(t("فهرست پخش (F6)")).clicked() {
                    self.show_playlist = !self.show_playlist;
                }
                if ui.selectable_label(self.show_subs, RichText::new("CC").size(15.0).strong()).on_hover_text(t("مرورگر زیرنویس (F7)")).clicked() {
                    self.show_subs = !self.show_subs;
                }
                if ui.selectable_label(self.show_prefs, icon("⚙")).on_hover_text(t("تنظیمات (F5)")).clicked() {
                    self.show_prefs = !self.show_prefs;
                }
                ui.add_space(6.0);
                ui.label(RichText::new(format!("{:>3.0}%", self.st.vol)).monospace());
                let mut v = self.st.vol;
                ui.spacing_mut().slider_width = 110.0;
                if ui.add(egui::Slider::new(&mut v, 0.0..=200.0).show_value(false).trailing_fill(true)).changed() {
                    self.set("volume", &format!("{v:.0}"));
                }
                let speaker = if self.st.mute || self.st.vol < 0.5 { "🔇" } else { "🔊" };
                if ui.button(icon(speaker)).on_hover_text(t("بی‌صدا (M)")).clicked() { self.cmd(&["cycle", "mute"]); }
            });
        });
    }

    // ---------- playlist (left side in RTL) ----------
    fn playlist_panel(&mut self, ui: &mut Ui) {
        rtl(ui, |ui| {
            ui.heading(t("فهرست پخش"));
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if ui.small_button("✖").clicked() { self.show_playlist = false; }
            });
        });
        rtl(ui, |ui| {
            if ui.button(t("+ فایل")).clicked() { self.open_dialog(true); }
            if ui.button(t("درهم")).clicked() { self.cmd(&["playlist-shuffle"]); self.refresh_soon(); }
            if ui.button(t("پاک کردن")).on_hover_text(t("فایل در حال پخش می‌ماند")).clicked() {
                self.cmd(&["playlist-clear"]);
                self.refresh_soon();
            }
        });
        ui.separator();
        if self.st.playlist.is_empty() {
            stack(ui, |ui| ui.label(RichText::new(t("فایل‌ها را اینجا رها کنید")).color(MUTED)));
        }
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| stack(ui, |ui| {
            for (i, (name, cur)) in self.st.playlist.clone().into_iter().enumerate() {
                let mut text = RichText::new(t(&format!("{}. {name}", i + 1)));
                if cur { text = text.color(ACCENT).strong(); }
                let r = ui.selectable_label(cur, text);
                let clicked = r.clicked();
                r.context_menu(|ui| stack(ui, |ui| {
                    if mi(ui, "پخش", "").clicked() { self.set("playlist-pos", &i.to_string()); ui.close_menu(); }
                    if mi(ui, "حذف از فهرست", "").clicked() {
                        self.cmd(&["playlist-remove", &i.to_string()]);
                        self.refresh_soon();
                        ui.close_menu();
                    }
                }));
                if clicked {
                    self.set("playlist-pos", &i.to_string());
                    self.refresh_soon();
                }
            }
        }));
    }

    // ---------- subtitle explorer (right side) ----------
    fn subs_panel(&mut self, ui: &mut Ui) {
        rtl(ui, |ui| {
            ui.heading(t("مرورگر زیرنویس"));
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if ui.small_button("✖").clicked() { self.show_subs = false; }
            });
        });

        // current line, large
        egui::Frame::none().fill(DEEP).rounding(8.0).inner_margin(egui::Margin::same(10.0)).show(ui, |ui| {
            let w = ui.available_width();
            ui.set_width(w);
            ui.set_min_height(70.0);
            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                if self.st.sub_text.is_empty() {
                    ui.label(RichText::new(t("در این لحظه زیرنویسی نیست")).color(MUTED));
                } else {
                    for line in fa::wrap_lines(&self.st.sub_text, 30) {
                        ui.add(egui::Label::new(RichText::new(line).size(20.0).color(Color32::WHITE)).wrap(false));
                    }
                }
            });
        });
        ui.add_space(4.0);

        let source = match (&self.sub_key, self.cues.is_empty()) {
            (None, _) => t("زیرنویسی انتخاب نشده"),
            (Some(k), false) => t(&format!("فایل: {}", file_name(k))),
            (Some(_), true) => t("زیرنویس داخلی: خط‌ها هنگام پخش جمع می‌شوند"),
        };
        stack(ui, |ui| ui.label(RichText::new(source).color(MUTED).small()));

        rtl(ui, |ui| {
            ui.checkbox(&mut self.follow, "");
            ui.label(t("دنبال کردن"));
            ui.add_space(6.0);
            let w = ui.available_width();
            ui.add(egui::TextEdit::singleline(&mut self.sub_search).hint_text(t("جستجو…")).desired_width(w));
        });
        rtl(ui, |ui| {
            ui.label(RichText::new(t(&format!("تأخیر: {:+.2} ثانیه", self.st.sub_delay))).color(MUTED));
            if ui.small_button("−").on_hover_text(t("زودتر")).clicked() { self.cmd(&["add", "sub-delay", "-0.1"]); }
            if ui.small_button("+").on_hover_text(t("دیرتر")).clicked() { self.cmd(&["add", "sub-delay", "0.1"]); }
            if ui.small_button(t("صفر")).clicked() { self.set("sub-delay", "0"); }
        });
        ui.separator();

        let now = self.st.pos - self.st.sub_delay;
        let q = self.sub_search.trim().to_lowercase();
        let list = if self.cues.is_empty() { &self.live } else { &self.cues };
        let rows: Vec<usize> = (0..list.len())
            .filter(|&i| q.is_empty() || list[i].text.to_lowercase().contains(&q))
            .collect();
        let cur = list.iter().position(|c| now >= c.start && now <= c.end);
        let cur_row = cur.and_then(|c| rows.iter().position(|&r| r == c));
        let jump = if self.follow && cur != self.last_cue { cur_row } else { None };
        let mut seek_to: Option<f64> = None;
        let mut sync_to: Option<f64> = None;
        let mut copy: Option<String> = None;

        let row_h = CUE_H + ui.spacing().item_spacing.y;
        let mut area = egui::ScrollArea::vertical().auto_shrink([false; 2]);
        if let Some(r) = jump {
            let h = ui.available_height();
            area = area.vertical_scroll_offset((r as f32 * row_h - h / 2.0 + CUE_H / 2.0).max(0.0));
        }
        area.show_rows(ui, CUE_H, rows.len(), |ui, range| {
            for ri in range {
                let i = rows[ri];
                let c = &list[i];
                let active = Some(i) == cur;
                let w = ui.available_width();
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, CUE_H), Sense::click());
                let p = ui.painter().with_clip_rect(rect);
                if active {
                    p.rect_filled(rect, 6.0, WINE);
                    p.rect_filled(egui::Rect::from_min_max(egui::pos2(rect.right() - 3.0, rect.top()), rect.right_bottom()), 0.0, ACCENT);
                } else if resp.hovered() {
                    p.rect_filled(rect, 6.0, WINDOW);
                }
                let right = rect.right() - 12.0;
                p.text(egui::pos2(right, rect.top() + 5.0), Align2::RIGHT_TOP, fmt_time(c.start),
                    FontId::monospace(13.0), if active { ACCENT } else { MUTED });
                for (k, line) in c.lines.iter().enumerate() {
                    p.text(egui::pos2(right, rect.top() + 23.0 + k as f32 * 18.0), Align2::RIGHT_TOP, line,
                        FontId::proportional(15.0), if active { Color32::WHITE } else { TEXT });
                }
                if resp.clicked() {
                    seek_to = Some(c.start);
                }
                resp.on_hover_text(t("کلیک: پرش به این خط  |  راست‌کلیک: همگام‌سازی"))
                    .context_menu(|ui| stack(ui, |ui| {
                        if mi(ui, "پرش به این خط", "").clicked() { seek_to = Some(c.start); ui.close_menu(); }
                        if mi(ui, "این خط همین الان گفته شد (همگام‌سازی)", "").clicked() { sync_to = Some(c.start); ui.close_menu(); }
                        if mi(ui, "کپی متن", "").clicked() { copy = Some(c.text.clone()); ui.close_menu(); }
                    }));
            }
        });
        self.last_cue = cur;

        if let Some(s) = seek_to {
            self.cmd(&["seek", &format!("{:.3}", s + self.st.sub_delay + 0.01), "absolute+exact"]);
        }
        if let Some(s) = sync_to {
            let d = self.st.pos - s;
            self.set("sub-delay", &format!("{d:.3}"));
            self.osd(format!("زیرنویس همگام شد ({d:+.2} ثانیه)"));
        }
        if let Some(text) = copy {
            ui.output_mut(|o| o.copied_text = text);
        }
    }

    // ---------- preferences ----------
    fn prefs_ui(&mut self, ui: &mut Ui) {
        rtl(ui, |ui| {
            ui.selectable_value(&mut self.tab, Tab::Video, t("تصویر"));
            ui.selectable_value(&mut self.tab, Tab::Audio, t("صدا"));
            ui.selectable_value(&mut self.tab, Tab::Subtitles, t("زیرنویس"));
            ui.selectable_value(&mut self.tab, Tab::Playback, t("پخش"));
        });
        ui.separator();
        match self.tab {
            Tab::Video => self.prefs_video(ui),
            Tab::Audio => self.prefs_audio(ui),
            Tab::Subtitles => self.prefs_subs(ui),
            Tab::Playback => self.prefs_playback(ui),
        }
    }

    fn prefs_video(&mut self, ui: &mut Ui) {
        for (i, (prop, label)) in COLOR_PROPS.iter().enumerate() {
            field(ui, label, |ui| {
                if ui.add(egui::Slider::new(&mut self.pf.color[i], -100..=100)).changed() {
                    self.set(prop, &self.pf.color[i].to_string());
                }
            });
        }
        field(ui, "زوم", |ui| {
            let mut z = self.pf.zoom;
            if ui.add(egui::Slider::new(&mut z, -2.0..=2.0).step_by(0.05)).changed() { self.set_zoom(z); }
        });
        field(ui, "بزرگ‌نمایی", |ui| {
            let mut scale = self.pf.scale.clone();
            egui::ComboBox::from_id_source("scaler").selected_text(scale.clone()).show_ui(ui, |ui| {
                for s in ["bilinear", "spline36", "ewa_lanczos", "ewa_lanczossharp"] {
                    ui.selectable_value(&mut scale, s.to_string(), s);
                }
            });
            if scale != self.pf.scale {
                self.set("scale", &scale);
                self.pf.scale = scale;
            }
        });
        field(ui, "نسبت تصویر", |ui| {
            for (label, v) in ASPECTS {
                if ui.selectable_label(self.pf.aspect == v, t(label)).clicked() { self.set_aspect(v); }
            }
        });
        ui.add_space(6.0);
        rtl(ui, |ui| {
            let mut d = self.pf.deint;
            if ui.checkbox(&mut d, "").changed() { self.toggle_deint(); }
            ui.label(t("حذف درهم‌رفتگی"));
            ui.add_space(10.0);
            let mut h = self.pf.hwdec;
            if ui.checkbox(&mut h, "").changed() { self.toggle_hw(); }
            ui.label(t("رمزگشایی سخت‌افزاری"));
        });
        rtl(ui, |ui| {
            if ui.button(t("چرخش ۹۰ درجه")).clicked() { self.rotate(); }
            if ui.button(t("بازنشانی تصویر")).clicked() { self.reset_color(); self.set_zoom(0.0); }
        });
    }

    fn prefs_audio(&mut self, ui: &mut Ui) {
        rtl(ui, |ui| {
            ui.label(t("پیش‌تنظیم:"));
            for (name, bands) in EQ_PRESETS {
                if ui.button(t(name)).clicked() { self.pf.eq = bands; self.af_dirty = true; }
            }
        });
        ui.add_space(6.0);
        // the EQ is a frequency graph, so it stays low → high, left → right
        ui.horizontal(|ui| {
            for i in 0..10 {
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("{:+.0}", self.pf.eq[i])).small().color(MUTED));
                    ui.spacing_mut().slider_width = 140.0;
                    let s = egui::Slider::new(&mut self.pf.eq[i], -12.0..=12.0)
                        .vertical()
                        .show_value(false)
                        .trailing_fill(true);
                    if ui.add(s).changed() { self.af_dirty = true; }
                    ui.label(RichText::new(EQ_LABELS[i]).small());
                });
            }
        });
        ui.add_space(8.0);
        field(ui, "بلندی صدا", |ui| {
            let mut v = self.st.vol;
            if ui.add(egui::Slider::new(&mut v, 0.0..=200.0).suffix("%")).changed() { self.set("volume", &format!("{v:.0}")); }
        });
        field(ui, "تأخیر صدا (ثانیه)", |ui| {
            let mut d = self.st.audio_delay;
            if ui.add(egui::DragValue::new(&mut d).speed(0.01)).changed() { self.set("audio-delay", &format!("{d:.3}")); }
        });
        field(ui, "کانال‌ها", |ui| {
            for (label, v) in [("خودکار", "auto-safe"), ("استریو", "stereo"), ("مونو", "mono")] {
                if ui.selectable_label(self.pf.channels == v, t(label)).clicked() {
                    self.pf.channels = v.to_string();
                    self.set("audio-channels", v);
                }
            }
        });
        rtl(ui, |ui| {
            if ui.checkbox(&mut self.pf.normalize, "").changed() { self.af_dirty = true; }
            ui.label(t("یکسان‌سازی بلندی صدا (حالت شب)"));
        });
    }

    fn prefs_subs(&mut self, ui: &mut Ui) {
        field(ui, "سبک آماده", |ui| {
            let mut changed = false;
            if ui.button(t("کلاسیک")).clicked() {
                self.pf.sub_color = Color32::WHITE; self.pf.sub_box = false; self.pf.sub_border = 2.5; changed = true;
            }
            if ui.button(t("زرد")).clicked() {
                self.pf.sub_color = Color32::from_rgb(0xFF, 0xE3, 0x4D); self.pf.sub_box = false; self.pf.sub_border = 3.0; changed = true;
            }
            if ui.button(t("کادر تیره")).clicked() {
                self.pf.sub_color = Color32::WHITE; self.pf.sub_box = true; self.pf.sub_border = 4.0; changed = true;
            }
            if changed { self.apply_sub_style(); }
        });
        field(ui, "فونت", |ui| {
            let mut font = self.pf.sub_font.clone();
            egui::ComboBox::from_id_source("subfont").selected_text(font.clone()).show_ui(ui, |ui| {
                for f in SUB_FONTS {
                    ui.selectable_value(&mut font, f.to_string(), f);
                }
            });
            if font != self.pf.sub_font { self.pf.sub_font = font; self.apply_sub_style(); }
            if ui.checkbox(&mut self.pf.sub_bold, t("ضخیم")).changed() { self.apply_sub_style(); }
        });
        field(ui, "اندازه فونت", |ui| {
            if ui.add(egui::Slider::new(&mut self.pf.sub_size, 20..=90)).changed() { self.apply_sub_style(); }
        });
        field(ui, "مقیاس", |ui| {
            if ui.add(egui::Slider::new(&mut self.pf.sub_scale, 0.3..=3.0)).changed() {
                self.set("sub-scale", &format!("{:.2}", self.pf.sub_scale));
            }
        });
        field(ui, "موقعیت عمودی", |ui| {
            if ui.add(egui::Slider::new(&mut self.pf.sub_pos, 0..=150)).changed() { self.set("sub-pos", &self.pf.sub_pos.to_string()); }
        });
        field(ui, "حاشیه", |ui| {
            if ui.add(egui::Slider::new(&mut self.pf.sub_border, 0.0..=8.0)).changed() { self.apply_sub_style(); }
        });
        field(ui, "رنگ متن", |ui| {
            if ui.color_edit_button_srgba(&mut self.pf.sub_color).changed() { self.apply_sub_style(); }
            if ui.checkbox(&mut self.pf.sub_box, t("کادر پشت متن")).changed() { self.apply_sub_style(); }
        });
        field(ui, "تأخیر (ثانیه)", |ui| {
            let mut d = self.st.sub_delay;
            if ui.add(egui::DragValue::new(&mut d).speed(0.05)).changed() { self.set("sub-delay", &format!("{d:.2}")); }
        });
        field(ui, "رمزگذاری فایل", |ui| {
            for (label, v) in [("فارسی/عربی", "cp1256"), ("UTF-8", "utf-8"), ("خودکار", "auto")] {
                if ui.selectable_label(self.pf.codepage == v, t(label)).clicked() {
                    self.pf.codepage = v.to_string();
                    self.set("sub-codepage", v);
                    self.cmd(&["sub-reload"]);
                }
            }
        });
        stack(ui, |ui| {
            ui.label(RichText::new(t("اگر زیرنویس فارسی به‌هم‌ریخته است، «فارسی/عربی» را انتخاب کنید.")).color(MUTED).small());
        });
    }

    fn prefs_playback(&mut self, ui: &mut Ui) {
        field(ui, "سرعت", |ui| {
            let mut s = self.st.speed;
            if ui.add(egui::Slider::new(&mut s, 0.25..=4.0).suffix("x")).changed() { self.set("speed", &format!("{s:.2}")); }
        });
        rtl(ui, |ui| {
            if ui.checkbox(&mut self.pf.resume, "").changed() {
                self.set("save-position-on-quit", if self.pf.resume { "yes" } else { "no" });
            }
            ui.label(t("ادامه پخش از آخرین موقعیت"));
        });
        rtl(ui, |ui| {
            let mut lf = self.st.loop_file;
            if ui.checkbox(&mut lf, "").changed() { self.set("loop-file", if lf { "inf" } else { "no" }); self.refresh_soon(); }
            ui.label(t("تکرار فایل"));
            ui.add_space(10.0);
            let mut lp = self.st.loop_playlist;
            if ui.checkbox(&mut lp, "").changed() { self.set("loop-playlist", if lp { "inf" } else { "no" }); self.refresh_soon(); }
            ui.label(t("تکرار فهرست پخش"));
        });
        ui.add_space(6.0);
        stack(ui, |ui| {
            ui.label(RichText::new(t("محل ذخیره عکس‌ها:")).color(MUTED));
            ui.label(RichText::new(self.cfg.join("screenshots").display().to_string()).monospace().small());
        });
    }

    fn info_ui(&self, ui: &mut Ui) {
        let p = &self.player;
        let s = |k: &str| p.str(k).unwrap_or_else(|| "–".into());
        let kbps = |k: &str| p.f64(k).map(|b| format!("{:.0} kb/s", b / 1000.0)).unwrap_or_else(|| "–".into());
        let res = match (p.i64("width"), p.i64("height")) {
            (Some(w), Some(h)) => format!("{w} × {h}"),
            _ => "–".into(),
        };
        let size = p.f64("file-size").map(|b| format!("{:.1} MB", b / 1_048_576.0)).unwrap_or_else(|| "–".into());
        let decoder = p
            .str("hwdec-current")
            .filter(|h| h != "no")
            .map(|h| format!("سخت‌افزاری ({h})"))
            .unwrap_or_else(|| "نرم‌افزاری".into());
        let rows = [
            ("فایل", s("filename")),
            ("قالب", s("file-format")),
            ("حجم", size),
            ("مدت", fmt_time(self.st.dur)),
            ("کدک تصویر", s("video-codec")),
            ("وضوح", res),
            ("نرخ فریم", p.f64("container-fps").map(|f| format!("{f:.3} fps")).unwrap_or_else(|| "–".into())),
            ("بیت‌ریت تصویر", kbps("video-bitrate")),
            ("رمزگشا", decoder),
            ("کدک صدا", s("audio-codec-name")),
            ("نرخ نمونه", p.i64("audio-params/samplerate").map(|r| format!("{r} Hz")).unwrap_or_else(|| "–".into())),
            ("کانال‌ها", s("audio-params/hr-channels")),
            ("بیت‌ریت صدا", kbps("audio-bitrate")),
        ];
        for (k, v) in rows {
            field(ui, k, |ui| { ui.label(RichText::new(t(&v)).color(Color32::WHITE)); });
        }
    }

    fn keys_ui(ui: &mut Ui) {
        let rows = [
            ("پخش / مکث", "Space"), ("تمام‌صفحه", "Enter / 2×Click"),
            ("۵ ثانیه جلو / عقب", "← →"), ("۳۰ ثانیه", "Ctrl+← →"), ("۶۰ ثانیه", "Alt+← →"),
            ("بلندی صدا", "↑ ↓ / Wheel"), ("بی‌صدا", "M"), ("سریع‌تر / کندتر / عادی", "C / X / Z"),
            ("فریم بعدی / قبلی", "F / D"), ("فایل قبلی / بعدی", "PgUp / PgDn"),
            ("تکرار A-B", "\\"), ("تأخیر زیرنویس", ", . /"),
            ("زیرنویس بعدی / پنهان", "Alt+S / Alt+H"), ("ترک صدای بعدی", "Ctrl+A"),
            ("روشنایی / کنتراست", "Q W / E R"), ("اشباع / ته‌رنگ", "T Y / U I"),
            ("بازنشانی تصویر", "Backspace"), ("نسبت تصویر", "A"), ("چرخش", "Ctrl+R"),
            ("عکس از صفحه", "Ctrl+E"), ("همیشه رو", "Ctrl+T"), ("اطلاعات فایل", "Tab"),
            ("تنظیمات / فهرست پخش / زیرنویس", "F5 / F6 / F7"),
        ];
        for (desc, key) in rows {
            rtl(ui, |ui| {
                ui.allocate_ui_with_layout(Vec2::new(210.0, 20.0), Layout::right_to_left(Align::Center), |ui| {
                    ui.set_min_width(210.0);
                    ui.label(t(desc));
                });
                ui.label(RichText::new(key).monospace().color(ACCENT));
            });
        }
    }

    fn windows(&mut self, ctx: &egui::Context) {
        if self.show_prefs {
            let mut open = true;
            egui::Window::new(t("تنظیمات")).open(&mut open).resizable(false).default_width(480.0)
                .show(ctx, |ui| self.prefs_ui(ui));
            self.show_prefs = open;
        }
        if self.show_info {
            let mut open = true;
            egui::Window::new(t("اطلاعات فایل")).open(&mut open).resizable(false).show(ctx, |ui| self.info_ui(ui));
            self.show_info = open;
        }
        if self.show_keys {
            let mut open = true;
            egui::Window::new(t("میانبرهای صفحه‌کلید")).open(&mut open).resizable(false).show(ctx, |ui| Self::keys_ui(ui));
            self.show_keys = open;
        }
        if self.show_url {
            let mut open = true;
            let mut go = false;
            egui::Window::new(t("باز کردن آدرس اینترنتی")).open(&mut open).collapsible(false).resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let r = ui.add(egui::TextEdit::singleline(&mut self.url)
                        .hint_text("https://…")
                        .desired_width(420.0));
                    if !r.has_focus() && self.url.is_empty() { r.request_focus(); }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    rtl(ui, |ui| {
                        if ui.button(t("پخش")).clicked() || enter { go = true; }
                        ui.label(RichText::new(t("برای یوتیوب، yt-dlp باید نصب باشد")).color(MUTED).small());
                    });
                });
            if go && !self.url.trim().is_empty() {
                let u = self.url.trim().to_string();
                self.open_paths(vec![u], false);
                self.url.clear();
            }
            self.show_url = open && !go;
        }
    }

    fn video_area(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let rect = ui.max_rect();
        let resp = ui.interact(rect, ui.id().with("video"), Sense::click());
        if self.st.idle {
            let p = ui.painter();
            p.text(rect.center() - Vec2::new(0.0, 22.0), Align2::CENTER_CENTER, "RustPlayer",
                FontId::proportional(56.0), ACCENT);
            p.text(rect.center() + Vec2::new(0.0, 28.0), Align2::CENTER_CENTER,
                t("فایل را اینجا رها کنید، کلیک کنید یا Ctrl+O را بزنید"), FontId::proportional(17.0), MUTED);
            if self.no_font_warning {
                p.text(rect.center() + Vec2::new(0.0, 60.0), Align2::CENTER_CENTER,
                    "Persian font not found: put Vazirmatn-Regular.ttf in the fonts folder", FontId::proportional(14.0), EMBER);
            }
        } else {
            let video = self.video.clone();
            let cb = egui_glow::CallbackFn::new(move |info, painter| {
                if let Ok(mut g) = video.0.lock() {
                    if let Some(v) = g.as_mut() {
                        let vp = info.viewport_in_pixels();
                        v.paint(
                            painter.gl(),
                            vp.left_px as i32,
                            vp.from_bottom_px as i32,
                            vp.width_px as i32,
                            vp.height_px as i32,
                        );
                    }
                }
            });
            ui.painter().add(egui::PaintCallback { rect, callback: Arc::new(cb) });
        }
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            ui.painter().rect_filled(rect.shrink(12.0), 12.0, Color32::from_black_alpha(170));
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, t("رها کنید تا پخش شود"),
                FontId::proportional(30.0), ACCENT);
        }
        if resp.double_clicked() {
            self.toggle_fs(ctx);
        } else if resp.clicked() {
            if self.st.idle { self.open_dialog(false); } else { self.toggle_pause(); }
        }
        if resp.hovered() {
            self.scroll_acc += ctx.input(|i| i.raw_scroll_delta.y);
            while self.scroll_acc.abs() >= 30.0 {
                let up = self.scroll_acc > 0.0;
                self.add_volume(if up { 5.0 } else { -5.0 });
                self.st.vol = (self.st.vol + if up { 5.0 } else { -5.0 }).clamp(0.0, 200.0);
                self.scroll_acc -= if up { 30.0 } else { -30.0 };
            }
        }
        resp.context_menu(|ui| {
            ui.set_min_width(220.0);
            stack(ui, |ui| {
                let label = if self.st.paused { "پخش" } else { "مکث" };
                if mi(ui, label, "Space").clicked() { self.toggle_pause(); ui.close_menu(); }
                ui.separator();
                self.menus(ui, ctx);
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();

        if self.st.title != self.win_title {
            self.win_title = self.st.title.clone();
            // OS title bars do their own bidi, so the raw title is used here.
            let title = if self.win_title.is_empty() { "RustPlayer".to_string() } else { format!("{} - RustPlayer", self.win_title) };
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }

        let dropped: Vec<String> = ctx.input(|i| {
            i.raw.dropped_files.iter()
                .filter_map(|f| f.path.as_ref().map(|p| p.to_string_lossy().into_owned()))
                .collect()
        });
        if !dropped.is_empty() {
            self.open_paths(dropped, false);
        }

        self.keys(ctx);
        if self.af_dirty && !ctx.input(|i| i.pointer.any_down()) {
            self.apply_af();
        }

        if ctx.input(|i| i.pointer.delta() != Vec2::ZERO || i.pointer.any_down()) {
            self.last_move = Instant::now();
        }
        let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        let show_ui = !fs || self.st.paused || self.st.idle || self.last_move.elapsed() < Duration::from_millis(2500);

        if show_ui {
            egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| self.menus(ui, ctx));
                });
            });
            egui::TopBottomPanel::bottom("controls")
                .frame(egui::Frame::none().fill(PANEL).inner_margin(egui::Margin::symmetric(12.0, 8.0)))
                .show(ctx, |ui| self.controls(ui, ctx));
            if self.show_playlist {
                egui::SidePanel::left("playlist")
                    .default_width(290.0)
                    .frame(egui::Frame::none().fill(PANEL).inner_margin(egui::Margin::same(10.0)))
                    .show(ctx, |ui| self.playlist_panel(ui));
            }
            if self.show_subs {
                egui::SidePanel::right("subs")
                    .default_width(360.0)
                    .min_width(280.0)
                    .frame(egui::Frame::none().fill(PANEL).inner_margin(egui::Margin::same(10.0)))
                    .show(ctx, |ui| self.subs_panel(ui));
            }
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| self.video_area(ui, ctx));

        self.windows(ctx);
        ctx.request_repaint_after(Duration::from_millis(200));
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if !self.st.idle {
            self.cmd(&["write-watch-later-config"]);
        }
        if let Ok(mut g) = self.video.0.lock() {
            if let Some(mut v) = g.take() {
                if let Some(gl) = gl {
                    v.destroy(gl);
                }
            }
        }
    }
}
