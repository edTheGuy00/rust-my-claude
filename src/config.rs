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

use std::path::PathBuf;

// ─── Style (global defaults) ─────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Style {
    /// `"powerline"` | `"rounded"` | `"plain"`.
    /// `"rounded"` = current look: rounded end-caps + `` dividers.
    /// `"powerline"` = same but no end-caps (flush edges).
    /// `"plain"` = no glyphs, space-separated, bg fills replaced by text `[##--]`.
    /// `"diamond"` (aliases `"bubbles"`, `"chips"`) = each segment is its own
    /// floating shape wrapped in `cap_left`/`cap_right` (round = bubbles; slant or
    /// flame caps give other shapes). `powerline_divider` is also configurable for
    /// slant/flame connected looks in `"powerline"` mode.
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
#[derive(Clone, Debug, Default)]
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
    "limit_5h",
    "limit_7d",
    "vim",
    "thinking",
    "effort",
    "pr",
    "session",
    "output_style",
    "version",
];

// ─── Parsing ─────────────────────────────────────────────────────────────────
//
// Themes/configs use a tiny, fixed TOML subset: `# comments`, `[table]` headers,
// and `key = value` where value is a double-quoted string, an integer, or a
// bool. No arrays, inline tables, floats, datetimes, or string escapes are used.
// A purpose-built parser for exactly this shape lets us drop the heavy `toml`
// crate dependency entirely (~115 KiB off the stripped binary). Anything it
// doesn't understand is skipped, matching the schema's "unknown tables/fields
// are silently ignored" contract.

/// A scalar config value in our subset.
enum Val {
    Str(String),
    Int(i64),
    Bool(bool),
}

/// Parse a value token: `"quoted string"`, `true`/`false`, or an integer.
/// Trailing whitespace/`# comments` after the value are ignored.
fn parse_value(s: &str) -> Option<Val> {
    if let Some(rest) = s.strip_prefix('"') {
        // String literal — up to the next quote (no escapes in our format).
        let end = rest.find('"')?;
        return Some(Val::Str(rest[..end].to_string()));
    }
    let token = s
        .split(|c: char| c.is_whitespace() || c == '#')
        .next()
        .unwrap_or("");
    match token {
        "true" => Some(Val::Bool(true)),
        "false" => Some(Val::Bool(false)),
        "" => None,
        _ => token.parse::<i64>().ok().map(Val::Int),
    }
}

/// Split the source into ordered `(table_name, [(key, value)])` sections.
fn parse_tables(s: &str) -> Vec<(String, Vec<(String, Val)>)> {
    let mut tables: Vec<(String, Vec<(String, Val)>)> = Vec::new();
    let mut cur: Option<usize> = None;
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(after) = line.strip_prefix('[') {
            // `[table]` header — take the name up to the closing bracket.
            let name = after.split(']').next().unwrap_or("").trim().to_string();
            tables.push((name, Vec::new()));
            cur = Some(tables.len() - 1);
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            if key.is_empty() {
                continue;
            }
            if let (Some(i), Some(val)) = (cur, parse_value(line[eq + 1..].trim())) {
                tables[i].1.push((key.to_string(), val));
            }
        }
    }
    tables
}

fn build_style(entries: &[(String, Val)]) -> Style {
    let mut st = Style::default();
    for (k, v) in entries {
        match (k.as_str(), v) {
            ("separator", Val::Str(s)) => st.separator = s.clone(),
            ("default_fg", Val::Int(n)) => st.default_fg = *n as u8,
            ("default_bg", Val::Int(n)) => st.default_bg = *n as u8,
            ("gauge_red_shift", Val::Bool(b)) => st.gauge_red_shift = *b,
            ("gauge_red_threshold", Val::Int(n)) => st.gauge_red_threshold = *n as u8,
            ("powerline_divider", Val::Str(s)) => st.powerline_divider = s.clone(),
            ("cap_left", Val::Str(s)) => st.cap_left = s.clone(),
            ("cap_right", Val::Str(s)) => st.cap_right = s.clone(),
            _ => {}
        }
    }
    st
}

fn build_component(entries: &[(String, Val)]) -> ComponentConfig {
    let mut cc = ComponentConfig::default();
    for (k, v) in entries {
        match (k.as_str(), v) {
            ("line", Val::Int(n)) => cc.line = Some(*n),
            ("order", Val::Int(n)) => cc.order = Some(*n),
            ("icon", Val::Str(s)) => cc.icon = Some(s.clone()),
            ("fg", Val::Int(n)) => cc.fg = Some(*n as u8),
            ("bg", Val::Int(n)) => cc.bg = Some(*n as u8),
            ("gauge", Val::Bool(b)) => cc.gauge = Some(*b),
            ("track", Val::Int(n)) => cc.track = Some(*n as u8),
            ("format", Val::Str(s)) => cc.format = Some(s.clone()),
            ("disabled", Val::Bool(b)) => cc.disabled = *b,
            _ => {}
        }
    }
    cc
}

fn parse_toml(s: &str) -> Option<Config> {
    let tables = parse_tables(s);
    let style = tables
        .iter()
        .find(|(name, _)| name == "style")
        .map(|(_, entries)| build_style(entries))
        .unwrap_or_default();
    // Iterate the known list in declaration order, picking up present tables —
    // identical ordering to the previous implementation.
    let mut components = Vec::new();
    for &name in KNOWN_COMPONENTS {
        if let Some((_, entries)) = tables.iter().find(|(n, _)| n == name) {
            components.push((name.to_string(), build_component(entries)));
        }
    }
    Some(Config { style, components })
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
