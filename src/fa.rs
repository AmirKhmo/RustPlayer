//! Persian/Arabic shaping + a small bidi pass.
//! egui has no text-shaping engine, so we convert letters to their contextual
//! presentation forms and reorder each line visually (RTL base, LTR runs kept).
use std::{cell::RefCell, collections::HashMap};

/// (base, isolated, final, initial, medial). initial == 0 means right-joining only.
const FORMS: &[(u32, u32, u32, u32, u32)] = &[
        (0x0621, 0xFE80, 0x0000, 0x0000, 0x0000), // hamza
    (0x0622, 0xFE81, 0xFE82, 0x0000, 0x0000), // alef with madda above
    (0x0623, 0xFE83, 0xFE84, 0x0000, 0x0000), // alef with hamza above
    (0x0624, 0xFE85, 0xFE86, 0x0000, 0x0000), // waw with hamza above
    (0x0625, 0xFE87, 0xFE88, 0x0000, 0x0000), // alef with hamza below
    (0x0626, 0xFE89, 0xFE8A, 0xFE8B, 0xFE8C), // yeh with hamza above
    (0x0627, 0xFE8D, 0xFE8E, 0x0000, 0x0000), // alef
    (0x0628, 0xFE8F, 0xFE90, 0xFE91, 0xFE92), // beh
    (0x0629, 0xFE93, 0xFE94, 0x0000, 0x0000), // teh marbuta
    (0x062A, 0xFE95, 0xFE96, 0xFE97, 0xFE98), // teh
    (0x062B, 0xFE99, 0xFE9A, 0xFE9B, 0xFE9C), // theh
    (0x062C, 0xFE9D, 0xFE9E, 0xFE9F, 0xFEA0), // jeem
    (0x062D, 0xFEA1, 0xFEA2, 0xFEA3, 0xFEA4), // hah
    (0x062E, 0xFEA5, 0xFEA6, 0xFEA7, 0xFEA8), // khah
    (0x062F, 0xFEA9, 0xFEAA, 0x0000, 0x0000), // dal
    (0x0630, 0xFEAB, 0xFEAC, 0x0000, 0x0000), // thal
    (0x0631, 0xFEAD, 0xFEAE, 0x0000, 0x0000), // reh
    (0x0632, 0xFEAF, 0xFEB0, 0x0000, 0x0000), // zain
    (0x0633, 0xFEB1, 0xFEB2, 0xFEB3, 0xFEB4), // seen
    (0x0634, 0xFEB5, 0xFEB6, 0xFEB7, 0xFEB8), // sheen
    (0x0635, 0xFEB9, 0xFEBA, 0xFEBB, 0xFEBC), // sad
    (0x0636, 0xFEBD, 0xFEBE, 0xFEBF, 0xFEC0), // dad
    (0x0637, 0xFEC1, 0xFEC2, 0xFEC3, 0xFEC4), // tah
    (0x0638, 0xFEC5, 0xFEC6, 0xFEC7, 0xFEC8), // zah
    (0x0639, 0xFEC9, 0xFECA, 0xFECB, 0xFECC), // ain
    (0x063A, 0xFECD, 0xFECE, 0xFECF, 0xFED0), // ghain
    (0x0641, 0xFED1, 0xFED2, 0xFED3, 0xFED4), // feh
    (0x0642, 0xFED5, 0xFED6, 0xFED7, 0xFED8), // qaf
    (0x0643, 0xFED9, 0xFEDA, 0xFEDB, 0xFEDC), // kaf
    (0x0644, 0xFEDD, 0xFEDE, 0xFEDF, 0xFEE0), // lam
    (0x0645, 0xFEE1, 0xFEE2, 0xFEE3, 0xFEE4), // meem
    (0x0646, 0xFEE5, 0xFEE6, 0xFEE7, 0xFEE8), // noon
    (0x0647, 0xFEE9, 0xFEEA, 0xFEEB, 0xFEEC), // heh
    (0x0648, 0xFEED, 0xFEEE, 0x0000, 0x0000), // waw
    (0x0649, 0xFEEF, 0xFEF0, 0x0000, 0x0000), // alef maksura
    (0x064A, 0xFEF1, 0xFEF2, 0xFEF3, 0xFEF4), // yeh
    (0x067E, 0xFB56, 0xFB57, 0xFB58, 0xFB59), // peh
    (0x0686, 0xFB7A, 0xFB7B, 0xFB7C, 0xFB7D), // tcheh
    (0x0698, 0xFB8A, 0xFB8B, 0x0000, 0x0000), // jeh
    (0x06A9, 0xFB8E, 0xFB8F, 0xFB90, 0xFB91), // keheh
    (0x06AF, 0xFB92, 0xFB93, 0xFB94, 0xFB95), // gaf
    (0x06CC, 0xFBFC, 0xFBFD, 0xFBFE, 0xFBFF), // farsi yeh
    (0x06C0, 0xFBA4, 0xFBA5, 0x0000, 0x0000), // heh with yeh above
];

/// Lam + alef ligatures: (alef variant, isolated, final)
const LAM_ALEF: &[(u32, u32, u32)] = &[
    (0x0622, 0xFEF5, 0xFEF6),
    (0x0623, 0xFEF7, 0xFEF8),
    (0x0625, 0xFEF9, 0xFEFA),
    (0x0627, 0xFEFB, 0xFEFC),
];

#[derive(PartialEq, Clone, Copy)]
enum J {
    Dual,
    Right,
    Causing,
    Transparent,
    None,
}

fn forms(c: u32) -> Option<&'static (u32, u32, u32, u32, u32)> {
    FORMS.iter().find(|f| f.0 == c)
}

fn is_transparent(c: u32) -> bool {
    (0x0610..=0x061A).contains(&c) || (0x064B..=0x065F).contains(&c) || c == 0x0670 || (0x06D6..=0x06ED).contains(&c)
}

fn joining(c: u32) -> J {
    if c == 0x0640 {
        return J::Causing;
    }
    if is_transparent(c) {
        return J::Transparent;
    }
    match forms(c) {
        Some(f) if f.3 != 0 => J::Dual,
        Some(f) if f.2 != 0 => J::Right,
        _ => J::None,
    }
}

fn shape(cs: &[u32]) -> Vec<u32> {
    let n = cs.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let c = cs[i];
        let Some(&(_, iso, fin, ini, med)) = forms(c) else {
            out.push(c);
            i += 1;
            continue;
        };
        // does the previous (non-transparent) letter connect towards us?
        let mut prev = false;
        let mut p = i;
        while p > 0 {
            p -= 1;
            match joining(cs[p]) {
                J::Transparent => continue,
                J::Dual | J::Causing => {
                    prev = true;
                    break;
                }
                _ => break,
            }
        }
        let mut j = i + 1;
        while j < n && joining(cs[j]) == J::Transparent {
            j += 1;
        }
        if c == 0x0644 && j < n {
            if let Some(&(_, l_iso, l_fin)) = LAM_ALEF.iter().find(|l| l.0 == cs[j]) {
                out.push(if prev { l_fin } else { l_iso });
                out.extend_from_slice(&cs[i + 1..j]);
                i = j + 1;
                continue;
            }
        }
        let next = ini != 0 && j < n && matches!(joining(cs[j]), J::Dual | J::Right | J::Causing);
        let f = match (prev, next) {
            (true, true) => med,
            (true, false) => if fin != 0 { fin } else { iso },
            (false, true) => ini,
            (false, false) => iso,
        };
        out.push(if f != 0 { f } else { c });
        i += 1;
    }
    out
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit() || ('\u{06F0}'..='\u{06F9}').contains(&c) || ('\u{0660}'..='\u{0669}').contains(&c)
}

pub fn is_rtl(c: char) -> bool {
    let o = c as u32;
    !is_digit(c)
        && ((0x0590..=0x08FF).contains(&o) || (0xFB1D..=0xFDFF).contains(&o) || (0xFE70..=0xFEFF).contains(&o))
}

fn is_ltr(c: char) -> bool {
    is_digit(c) || (c.is_alphanumeric() && !is_rtl(c))
}

fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}

fn visual_line(s: &str) -> String {
    if !s.chars().any(is_rtl) {
        return s.to_string();
    }
    let cps: Vec<u32> = s.chars().map(|c| c as u32).collect();
    let chars: Vec<char> = shape(&cps)
        .into_iter()
        .filter_map(char::from_u32)
        .filter(|c| !matches!(c, '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}'))
        .collect();
    let cls: Vec<u8> = chars
        .iter()
        .map(|&c| if is_rtl(c) { b'R' } else if is_ltr(c) { b'L' } else { b'N' })
        .collect();
    let n = chars.len();
    let mut dir = cls.clone();
    for i in 0..n {
        if cls[i] == b'N' {
            let l = cls[..i].iter().rev().find(|&&d| d != b'N').copied().unwrap_or(b'R');
            let r = cls[i + 1..].iter().find(|&&d| d != b'N').copied().unwrap_or(b'R');
            dir[i] = if l == b'L' && r == b'L' { b'L' } else { b'R' };
        }
    }
    let mut runs: Vec<(u8, Vec<char>)> = Vec::new();
    for (c, d) in chars.into_iter().zip(dir) {
        match runs.last_mut() {
            Some((rd, v)) if *rd == d => v.push(c),
            _ => runs.push((d, vec![c])),
        }
    }
    let mut out = String::with_capacity(n * 3);
    for (d, v) in runs.into_iter().rev() {
        if d == b'R' {
            out.extend(v.into_iter().rev().map(mirror));
        } else {
            out.extend(v);
        }
    }
    out
}

/// Logical text -> visual text ready for egui.
pub fn visual(s: &str) -> String {
    s.split('\n').map(visual_line).collect::<Vec<_>>().join("\n")
}

thread_local! {
    static CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Cached `visual`, used for every UI string.
pub fn t(s: &str) -> String {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 5000 {
            c.clear();
        }
        c.entry(s.to_string()).or_insert_with(|| visual(s)).clone()
    })
}

/// Word-wrap logical text to `max` chars per line, then convert each line to visual order.
pub fn wrap_lines(s: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for para in s.lines() {
        let mut cur = String::new();
        for w in para.split_whitespace() {
            if !cur.is_empty() && cur.chars().count() + 1 + w.chars().count() > max {
                lines.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
    }
    lines.iter().map(|l| visual_line(l)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shapes_persian() {
        let v: Vec<u32> = visual("تنظیمات").chars().map(|c| c as u32).collect();
        assert_eq!(v, vec![0xFE95, 0xFE8E, 0xFEE4, 0xFBFF, 0xFEC8, 0xFEE8, 0xFE97]);
        assert!(visual("باز کردن فایل (Ctrl+O)").starts_with("(Ctrl+O) "));
    }
}
