//! Subtitle file reader for the subtitle explorer (SRT, VTT, ASS/SSA).
//! Handles UTF-8, UTF-16 and Windows-1256 (common for Persian .srt files).
use crate::fa;
use std::{cmp::Ordering, fs, path::Path};

#[derive(Clone)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
    /// Pre-shaped, wrapped lines for drawing (max 2).
    pub lines: Vec<String>,
}

impl Cue {
    pub fn new(start: f64, end: f64, text: String) -> Self {
        let mut lines = fa::wrap_lines(&text, 38);
        if lines.len() > 2 {
            lines.truncate(2);
            lines[1] = format!("… {}", lines[1]);
        }
        Self { start, end, text, lines }
    }
}

pub fn load(path: &str) -> Vec<Cue> {
    let Ok(bytes) = fs::read(path) else { return Vec::new() };
    let text = decode(&bytes);
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut cues = if ext == "ass" || ext == "ssa" { parse_ass(&text) } else { parse_srt(&text) };
    cues.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(Ordering::Equal));
    cues
}

fn decode(b: &[u8]) -> String {
    if b.starts_with(&[0xFF, 0xFE]) {
        return encoding_rs::UTF_16LE.decode(b).0.into_owned();
    }
    if b.starts_with(&[0xFE, 0xFF]) {
        return encoding_rs::UTF_16BE.decode(b).0.into_owned();
    }
    let b = b.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(b);
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => encoding_rs::WINDOWS_1256.decode(b).0.into_owned(),
    }
}

fn parse_time(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', ".");
    let parts: Vec<&str> = s.split(':').collect();
    let (h, m, sec) = match parts.as_slice() {
        [h, m, s] => (h.parse::<f64>().ok()?, m.parse::<f64>().ok()?, s.parse::<f64>().ok()?),
        [m, s] => (0.0, m.parse::<f64>().ok()?, s.parse::<f64>().ok()?),
        _ => return None,
    };
    Some(h * 3600.0 + m * 60.0 + sec)
}

fn strip_tags(s: &str) -> String {
    let (mut angle, mut brace) = (false, false);
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' if !brace => angle = true,
            '>' if angle => angle = false,
            '{' if !angle => brace = true,
            '}' if brace => brace = false,
            _ if !angle && !brace => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

fn parse_srt(t: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut lines = t.lines().peekable();
    while let Some(l) = lines.next() {
        let Some((a, b)) = l.split_once("-->") else { continue };
        let start = parse_time(a);
        let end = parse_time(b.split_whitespace().next().unwrap_or(""));
        let mut text = Vec::new();
        while let Some(n) = lines.peek() {
            if n.trim().is_empty() {
                break;
            }
            let s = strip_tags(n.trim());
            if !s.is_empty() {
                text.push(s);
            }
            lines.next();
        }
        if let (Some(s), Some(e)) = (start, end) {
            if !text.is_empty() {
                cues.push(Cue::new(s, e, text.join("\n")));
            }
        }
    }
    cues
}

fn parse_ass(t: &str) -> Vec<Cue> {
    t.lines()
        .filter_map(|l| {
            let rest = l.trim_start().strip_prefix("Dialogue:")?;
            let f: Vec<&str> = rest.splitn(10, ',').collect();
            if f.len() < 10 {
                return None;
            }
            let text = f[9].replace("\\N", "\n").replace("\\n", "\n").replace("\\h", " ");
            let text = strip_tags(&text);
            if text.is_empty() {
                return None;
            }
            Some(Cue::new(parse_time(f[1])?, parse_time(f[2])?, text))
        })
        .collect()
}
