//! Bundled theme registry.
//!
//! A theme IS a complete config.toml. Themes are compiled into the binary via
//! `include_str!` so the CLI works with no network access. The real .toml files
//! live under the top-level `themes/` directory and are editable; this module
//! references them at compile time.
//!
//! To add a new theme:
//!   1. Create `themes/<name>.toml` following the annotated format in
//!      themes/powerline.toml.
//!   2. Add one line to the `THEMES` array below.

/// A bundled theme: name, one-line description, and the raw TOML string.
pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub toml: &'static str,
}

pub static THEMES: &[Theme] = &[
    Theme {
        name: "powerline",
        description: "Classic powerline pills, gauges, red-shift (the original look)",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/powerline.toml"
        )),
    },
    Theme {
        name: "minimal",
        description: "ASCII-safe, plain separators, model + context + cost only",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/minimal.toml"
        )),
    },
    Theme {
        name: "nord",
        description: "Cool arctic palette, rounded pills",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/nord.toml"
        )),
    },
];

/// Look up a theme by name (case-sensitive).
pub fn get(name: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.name == name)
}

/// The default theme TOML string, used when config is missing or invalid.
pub fn default_toml() -> &'static str {
    get("powerline").expect("powerline theme must be bundled").toml
}
