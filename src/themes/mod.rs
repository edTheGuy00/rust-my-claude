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
    Theme {
        name: "agnoster",
        description: "Classic agnoster powerline — blue/cyan/green/red segments",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/agnoster.toml"
        )),
    },
    Theme {
        name: "dracula",
        description: "Dracula — purple, pink, cyan, green on dark slate",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/dracula.toml"
        )),
    },
    Theme {
        name: "gruvbox",
        description: "Gruvbox — warm retro orange/yellow/aqua on dark brown",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/gruvbox.toml"
        )),
    },
    Theme {
        name: "tokyonight",
        description: "Tokyo Night — deep blue/purple/cyan night palette",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/tokyonight.toml"
        )),
    },
    Theme {
        name: "catppuccin",
        description: "Catppuccin Mocha — soft pastel mauve/pink/teal/green",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/catppuccin.toml"
        )),
    },
    Theme {
        name: "solarized-dark",
        description: "Solarized Dark — base03 with yellow/blue/cyan accents",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/solarized-dark.toml"
        )),
    },
    Theme {
        name: "solarized-light",
        description: "Solarized Light — light parchment bg, dark text",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/solarized-light.toml"
        )),
    },
    Theme {
        name: "monokai",
        description: "Monokai — vivid green/pink/orange/cyan on charcoal",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/monokai.toml"
        )),
    },
    Theme {
        name: "onedark",
        description: "One Dark — Atom-style blue/green/red/purple",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/onedark.toml"
        )),
    },
    Theme {
        name: "rose-pine",
        description: "Rosé Pine — muted rose/gold/pine/foam",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/rose-pine.toml"
        )),
    },
    Theme {
        name: "cyberpunk",
        description: "Cyberpunk — neon magenta + cyan, maximum contrast",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/cyberpunk.toml"
        )),
    },
    Theme {
        name: "matrix",
        description: "Matrix — monochrome green-on-black, plain separators",
        toml: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/themes/matrix.toml"
        )),
    },
    Theme {
        name: "bubbles",
        description: "Floating rounded bubbles (diamond style), colourful backgrounds",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/bubbles.toml")),
    },
    Theme {
        name: "chips",
        description: "Floating chips on a dark base, coloured labels (diamond style)",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/chips.toml")),
    },
    Theme {
        name: "cert",
        description: "Floating segments with slanted ice caps (diamond style)",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/cert.toml")),
    },
    Theme {
        name: "flame",
        description: "Connected chain with flame-shaped dividers, ember palette",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/flame.toml")),
    },
    Theme {
        name: "slant",
        description: "Connected chain with angled/slanted dividers, cool palette",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/slant.toml")),
    },
    Theme {
        name: "emodipt",
        description: "Minimalist plain text, no backgrounds (transparent prompt)",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/emodipt.toml")),
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
