//! Powerline rendering: Pill, gauge fill, color math, separator modes.
//!
//! The core rendering logic is preserved verbatim from the original main.rs,
//! with two changes:
//!   - `shift_to_red` reads threshold/enable from `&Style` instead of hardcoded
//!     constants.
//!   - `Pill` fields use `String` (owned) instead of `&'static str` so config-
//!     driven icons and text work without lifetime gymnastics.

use crate::config::Style;

pub const RESET: &str = "\x1b[0m";

// ─── Colour math ─────────────────────────────────────────────────────────────

/// xterm-256 palette index → approximate RGB.
pub fn xterm_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0..=15 => {
            const BASE: [(u8, u8, u8); 16] = [
                (0, 0, 0),
                (205, 0, 0),
                (0, 205, 0),
                (205, 205, 0),
                (0, 0, 238),
                (205, 0, 205),
                (0, 205, 205),
                (229, 229, 229),
                (127, 127, 127),
                (255, 0, 0),
                (0, 255, 0),
                (255, 255, 0),
                (92, 92, 255),
                (255, 0, 255),
                (0, 255, 255),
                (255, 255, 255),
            ];
            BASE[n as usize]
        }
        16..=231 => {
            let n = n - 16;
            let lvl = |v: u8| {
                if v == 0 {
                    0
                } else {
                    55 + v * 40
                }
            };
            (lvl(n / 36), lvl((n % 36) / 6), lvl(n % 6))
        }
        _ => {
            let v = 8 + (n - 232) * 10;
            (v, v, v)
        }
    }
}

/// RGB → nearest index in the 6×6×6 xterm colour cube (indices 16–231).
pub fn nearest_xterm(r: u8, g: u8, b: u8) -> u8 {
    const LV: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let idx = |v: u8| {
        let (mut best, mut bd) = (0u8, u16::MAX);
        for (i, &l) in LV.iter().enumerate() {
            let d = (l as i16 - v as i16).unsigned_abs();
            if d < bd {
                bd = d;
                best = i as u8;
            }
        }
        best
    };
    16 + 36 * idx(r) + 6 * idx(g) + idx(b)
}

/// Blend `base` color toward red as `pct` rises past the configured threshold.
/// Unchanged below threshold; fully red at 100%.
pub fn shift_to_red(base: u8, pct: f64, style: &Style) -> u8 {
    if !style.gauge_red_shift {
        return base;
    }
    let threshold = style.gauge_red_threshold as f64;
    let t = ((pct - threshold) / (100.0 - threshold)).clamp(0.0, 1.0);
    if t == 0.0 {
        return base;
    }
    let (br, bg, bb) = xterm_rgb(base);
    let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
    nearest_xterm(lerp(br, 255), lerp(bg, 0), lerp(bb, 0))
}

// ─── Pill ─────────────────────────────────────────────────────────────────────

/// One Powerline segment: icon + text on a solid or gauge-filled color block.
pub struct Pill {
    pub icon: String,
    pub text: String,
    pub fg: u8,
    pub bg: u8,
    /// `Some((track_color, percent))` for gauge fill; `None` for solid.
    pub fill: Option<(u8, f64)>,
}

impl Pill {
    /// A solid-colour pill.
    pub fn new(icon: impl Into<String>, text: impl Into<String>, fg: u8, bg: u8) -> Self {
        Pill { icon: icon.into(), text: text.into(), fg, bg, fill: None }
    }

    /// A gauge pill whose background fills left-to-right by `pct`.
    /// The filled hue bleeds toward red as `pct` approaches 100%.
    pub fn gauge(
        icon: impl Into<String>,
        text: impl Into<String>,
        pct: f64,
        filled: u8,
        track: u8,
        fg: u8,
        style: &Style,
    ) -> Self {
        Pill {
            icon: icon.into(),
            text: text.into(),
            fg,
            bg: shift_to_red(filled, pct, style),
            fill: Some((track, pct)),
        }
    }

    /// Render the body; return `(leading_bg, ansi_string, trailing_bg)`.
    pub fn body(&self) -> (u8, String, u8) {
        let label = if self.icon.is_empty() {
            format!(" {} ", self.text)
        } else {
            format!(" {} {} ", self.icon, self.text)
        };
        match self.fill {
            None => (
                self.bg,
                format!("\x1b[48;5;{};38;5;{}m{label}", self.bg, self.fg),
                self.bg,
            ),
            Some((track, pct)) => {
                let chars: Vec<char> = label.chars().collect();
                let total = chars.len().max(1);
                let filled =
                    (((pct / 100.0) * total as f64).round() as usize).min(total);
                let mut s = String::new();
                let mut cur: i16 = -1;
                for (i, c) in chars.iter().enumerate() {
                    let bg = if i < filled { self.bg } else { track };
                    if bg as i16 != cur {
                        s.push_str(&format!(
                            "\x1b[48;5;{};38;5;{}m",
                            bg, self.fg
                        ));
                        cur = bg as i16;
                    }
                    s.push(*c);
                }
                let leading = if filled >= 1 { self.bg } else { track };
                let trailing = if filled >= total { self.bg } else { track };
                (leading, s, trailing)
            }
        }
    }
}

// ─── Line rendering ──────────────────────────────────────────────────────────

/// Render a slice of pills into one ANSI line string using the configured
/// separator mode. Returns an empty string for an empty pill list.
pub fn render_line(pills: &[Pill], style: &Style) -> String {
    if pills.is_empty() {
        return String::new();
    }
    match style.separator.as_str() {
        "plain" => render_plain(pills),
        "diamond" | "bubbles" | "chips" => render_diamond(pills, style),
        "powerline" => render_powerline(pills, style, false),
        _ => render_powerline(pills, style, true), // "rounded" and default
    }
}

/// Diamond / bubble / chip mode: each pill is a self-contained shape wrapped in
/// its own leading + trailing cap (`cap_left`/`cap_right`), drawn in the pill's
/// own colour on the terminal background. Pills float, separated by a space.
/// Round caps (`` / ``) give bubbles; slant (`` / ``) or flame
/// (`` / ``) caps give other shapes — all driven by the theme's cap glyphs.
fn render_diamond(pills: &[Pill], style: &Style) -> String {
    let cap_l: char = style.cap_left.chars().next().unwrap_or('\u{e0b6}');
    let cap_r: char = style.cap_right.chars().next().unwrap_or('\u{e0b4}');
    let mut parts: Vec<String> = Vec::with_capacity(pills.len());
    for pill in pills {
        let (lead, body, trail) = pill.body();
        parts.push(format!(
            "\x1b[38;5;{lead}m{cap_l}{body}{RESET}\x1b[38;5;{trail}m{cap_r}{RESET}"
        ));
    }
    parts.join(" ")
}

/// Rounded caps + `` dividers (current look). `caps` controls whether
/// rounded end-caps are emitted (true = rounded, false = powerline/flush).
fn render_powerline(pills: &[Pill], style: &Style, caps: bool) -> String {
    let bodies: Vec<(u8, String, u8)> = pills.iter().map(Pill::body).collect();
    let sep: char = style.powerline_divider.chars().next().unwrap_or('\u{e0b0}');
    let cap_l: char = style.cap_left.chars().next().unwrap_or('\u{e0b6}');
    let cap_r: char = style.cap_right.chars().next().unwrap_or('\u{e0b4}');

    let mut out = if caps {
        format!("\x1b[38;5;{}m{}", bodies[0].0, cap_l)
    } else {
        String::new()
    };

    for (i, (_, body, trailing)) in bodies.iter().enumerate() {
        out.push_str(body);
        match bodies.get(i + 1) {
            Some((next_lead, _, _)) => {
                out.push_str(&format!(
                    "\x1b[48;5;{};38;5;{}m{}",
                    next_lead, trailing, sep
                ));
            }
            None => {
                if caps {
                    out.push_str(&format!(
                        "{RESET}\x1b[38;5;{}m{cap_r}{RESET}",
                        trailing
                    ));
                } else {
                    out.push_str(RESET);
                }
            }
        }
    }
    out
}

/// Plain mode: fg-colored ` icon text ` joined by a single space, no bg fill.
/// Gauges show `[####----] pct%` text representation instead of bg fill.
fn render_plain(pills: &[Pill]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for pill in pills {
        let text = match pill.fill {
            Some((_, pct)) => {
            // ASCII gauge bar: [####----]
            let total = 8usize;
            let filled = ((pct / 100.0) * total as f64).round() as usize;
            let bar: String = (0..total)
                .map(|i| if i < filled { '#' } else { '-' })
                .collect();
            if pill.icon.is_empty() {
                format!("[{bar}] {}", pill.text)
            } else {
                format!("{} [{bar}] {}", pill.icon, pill.text)
            }
            }
            None => {
            if pill.icon.is_empty() {
                pill.text.clone()
            } else {
                format!("{} {}", pill.icon, pill.text)
            }
            }
        };
        parts.push(format!("\x1b[38;5;{}m{text}{RESET}", pill.fg));
    }
    parts.join(" ")
}

// ─── Full statusline ─────────────────────────────────────────────────────────

/// Render a complete statusline: one ANSI string per output line, joined by
/// `\n`, with a final `\n`. Empty lines are rendered as empty strings (the
/// `println!` in main will still emit a blank line).
pub fn render_lines(lines: &[Vec<Pill>], style: &Style) -> String {
    let rendered: Vec<String> = lines.iter().map(|ps| render_line(ps, style)).collect();
    let mut out = rendered.join("\n");
    out.push('\n');
    out
}
