//! TOML config model, resolution, and loading.
//!
//! # Config file resolution precedence
//!
//! 1. `$RUST_MY_CLAUDE_CONFIG` — if the env var is set, that exact path is used.
//! 2. `$XDG_CONFIG_HOME/rust-my-claude/config.toml` — if `XDG_CONFIG_HOME` is set.
//! 3. `~/.config/rust-my-claude/config.toml` — the XDG default location.
//!
//! If the resolved file is missing or fails to parse, the bundled `powerline`
//! theme is used as a fallback. **The statusline never panics.**
//!
//! # Schema overview
//!
//! A config is one optional `[style]` table plus any number of named component
//! tables. **Presence of a component table = that component is shown; absence =
//! hidden.** Unknown tables and unknown fields inside known tables are silently
//! ignored. A component table can set `disabled = true` to suppress it without
//! removing the table.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Style (global defaults) ─────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Style {
    /// `"powerline"` | `"rounded"` | `"plain"`.
    /// `"rounded"` = current look: rounded end-caps + `` dividers.
    /// `"powerline"` = same but no end-caps (flush edges).
    /// `"plain"` = no glyphs, space-separated, bg fills replaced by text `[##--]`.
    pub separator: String,
    /// Fallback text color for any component that omits `fg`.
    pub default_fg: u8,
    /// Fallback pill bg for any component that omits `bg`.
    pub default_bg: u8,
    /// Enable hue-toward-red as a gauge nears 100%.
    pub gauge_red_shift: bool,
    /// Percent at which red-shift begins (matches original 60%).
    pub gauge_red_threshold: u8,
    /// The hard powerline divider glyph (between pills).
    pub powerline_divider: String,
    /// Rounded left end-cap.
    pub cap_left: String,
    /// Rounded right end-cap.
    pub cap_right: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            separator: "rounded".into(),
            default_fg: 231,
            default_bg: 238,
            gauge_red_shift: true,
            gauge_red_threshold: 60,
            powerline_divider: "\u{e0b0}".into(),
            cap_left: "\u{e0b6}".into(),
            cap_right: "\u{e0b4}".into(),
        }
    }
}

// ─── Per-component config ────────────────────────────────────────────────────

/// Config fields common to every component table.
/// Missing fields get sane defaults. Unknown fields are silently ignored.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct ComponentConfig {
    /// 1-based output line; out-of-range values are clamped to 1.
    pub line: Option<i64>,
    /// Left-to-right position within the line; ties broken by source order.
    pub order: Option<i64>,
    /// Nerd Font icon glyph; empty string = no icon.
    pub icon: Option<String>,
    /// Text color (256-color index).
    pub fg: Option<u8>,
    /// Pill background (or gauge filled-color).
    pub bg: Option<u8>,
    /// Enable gauge fill (only meaningful for gauge-capable components).
    pub gauge: Option<bool>,
    /// Gauge unfilled-track color.
    pub track: Option<u8>,
    /// Format template string; tokens vary per component.
    pub format: Option<String>,
    /// If true, treat the component as absent even though the table is present.
    #[serde(default)]
    pub disabled: bool,
}

impl ComponentConfig {
    pub fn line(&self) -> u8 {
        let v = self.line.unwrap_or(1);
        if !(1..=2).contains(&v) { 1 } else { v as u8 }
    }
    pub fn order(&self) -> i64 {
        self.order.unwrap_or(0)
    }
    pub fn icon_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.icon.as_deref().unwrap_or(default)
    }
    pub fn fg_or(&self, default: u8) -> u8 {
        self.fg.unwrap_or(default)
    }
    pub fn bg_or(&self, default: u8) -> u8 {
        self.bg.unwrap_or(default)
    }
    pub fn gauge_or(&self, default: bool) -> bool {
        self.gauge.unwrap_or(default)
    }
    pub fn format_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.format.as_deref().unwrap_or(default)
    }
}

// ─── Config (the whole file) ─────────────────────────────────────────────────

/// Intermediate deserialization target: a `[style]` table plus a catch-all
/// map of every other table. We then pull out known component keys by name.
#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    style: Style,
    #[serde(flatten)]
    rest: HashMap<String, toml::Value>,
}

/// The parsed, validated config.
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub style: Style,
    /// Named component tables, in source order.
    pub components: Vec<(String, ComponentConfig)>,
}

// ─── Known component names ────────────────────────────────────────────────────

const KNOWN_COMPONENTS: &[&str] = &[
    "model",
    "directory",
    "project_dir",
    "git",
    "repo",
    "context",
    "context_tokens",
    "cost",
    "duration",
    "limits",
    "vim",
    "thinking",
    "effort",
    "pr",
    "session",
    "output_style",
    "version",
];

// ─── Parsing ─────────────────────────────────────────────────────────────────

fn parse_toml(s: &str) -> Option<Config> {
    let raw: RawConfig = toml::from_str(s).ok()?;
    let mut components = Vec::new();
    // Preserve the order components appear by sorting unknown keys to the end;
    // since toml::Value deserialization doesn't guarantee insertion order we
    // iterate the known list in declaration order and pick up what's present.
    for &name in KNOWN_COMPONENTS {
        if let Some(val) = raw.rest.get(name) {
            // Ignore unknown fields gracefully by round-tripping through Value.
            if let Ok(cc) = val.clone().try_into::<ComponentConfig>() {
                components.push((name.to_string(), cc));
            }
        }
    }
    Some(Config { style: raw.style, components })
}

// ─── Resolution ──────────────────────────────────────────────────────────────

/// Resolve the config file path according to precedence rules (see module doc).
pub fn resolve_path() -> PathBuf {
    if let Ok(p) = std::env::var("RUST_MY_CLAUDE_CONFIG") {
        return PathBuf::from(p);
    }
    let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".into());
        PathBuf::from(home).join(".config")
    };
    base.join("rust-my-claude").join("config.toml")
}

/// Load and parse the config. Never fails: falls back to the bundled
/// `powerline` theme on any missing file or parse error.
pub fn load() -> Config {
    let path = resolve_path();
    if let Ok(s) = std::fs::read_to_string(&path)
        && let Some(cfg) = parse_toml(&s)
    {
        return cfg;
    }
    // Fall back to bundled powerline theme — this must always parse.
    parse_toml(crate::themes::default_toml()).expect("bundled powerline theme must parse")
}

/// Parse a raw TOML string (e.g. a bundled theme). Returns None on error.
pub fn parse(s: &str) -> Option<Config> {
    parse_toml(s)
}
