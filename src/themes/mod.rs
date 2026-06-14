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
    Theme {
        name: "jandedobbeleer",
        description: "Mixed diamond+powerline with round bubble caps (E0B6/E0B4) and standard chevron dividers; vivid palette of hot-pink directory, purple model, and yellow git on dark backgrounds.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/jandedobbeleer.toml")),
    },
    Theme {
        name: "powerlevel10k-rainbow",
        description: "Powerline chevron dividers with GNOME Tango palette: cobalt-blue directory, forest-green model, light-grey repo on near-white default background.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/powerlevel10k-rainbow.toml")),
    },
    Theme {
        name: "powerlevel10k-lean",
        description: "Pure plain/transparent style with no backgrounds; cyan path, yellow git, and teal model on terminal background — ultra-minimal lean look.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/powerlevel10k-lean.toml")),
    },
    Theme {
        name: "powerlevel10k-modern",
        description: "Round diamond bubbles with rightward chevron dividers (E0B4); vivid blue directory, cyan-teal model, and lime-yellow cost on dark default background.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/powerlevel10k-modern.toml")),
    },
    Theme {
        name: "paradox",
        description: "Powerline with pastel palette: sky-blue directory, warm-yellow model, mint-green git, violet repo — near-black text on classic right-arrow dividers.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/paradox.toml")),
    },
    Theme {
        name: "agnosterplus",
        description: "Clean three-segment powerline: sky-blue path, white model, mint-green git — near-black text with blue accent on line 2, matching the agnosterplus original.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/agnosterplus.toml")),
    },
    Theme {
        name: "agnoster-minimal",
        description: "Plain/transparent with VS Code blue (#007ACC → xterm 32) as the sole accent: no backgrounds, blue directory and context, steel-blue model and cost.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/agnoster-minimal.toml")),
    },
    Theme {
        name: "robbyrussell",
        description: "Plain/transparent oh-my-zsh classic: teal directory, sage-green model, rosy-red git, yellow context gauge — no backgrounds, pure colour-coded text.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/robbyrussell.toml")),
    },
    Theme {
        name: "sorin",
        description: "Plain/transparent style with Solarized-adjacent palette: steel-blue directory, yellow-ochre git, red cost accent, white default text — no backgrounds, minimal noise.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/sorin.toml")),
    },
    Theme {
        name: "pure",
        description: "Minimalist plain style in Nord palette: steel-blue directory, mauve model, dim-grey git, muted cyan context gauge, sage-green cost — calm two-line layout.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/pure.toml")),
    },
    Theme {
        name: "lambda",
        description: "Ultra-minimal plain style: bold crimson red for directory and context gauge, off-white default text on transparent background — stark two-colour lambda aesthetic.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/lambda.toml")),
    },
    Theme {
        name: "star",
        description: "Plain style in One Dark palette: teal directory, purple model and git, sage-green cost, coral/pink accents — vibrant multi-colour transparent layout.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/star.toml")),
    },
    Theme {
        name: "spaceship",
        description: "Plain/transparent style with no backgrounds; cyan directory, pink git, light-yellow model, green cost — faithful to the spaceship-prompt minimalist aesthetic.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/spaceship.toml")),
    },
    Theme {
        name: "atomic",
        description: "Diamond bubbles with rounded caps; vibrant cobalt-blue, orange, and yellow palette on dark backgrounds with a teal/cyan accent line.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/atomic.toml")),
    },
    Theme {
        name: "cobalt2",
        description: "Diamond bubbles with rounded caps; Wes Bos's cobalt2 palette of deep cobalt blue, electric green, and golden yellow on near-black.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/cobalt2.toml")),
    },
    Theme {
        name: "night-owl",
        description: "Powerline style with Sarah Drasner's Night Owl palette: periwinkle directory, lime-green model, warm-yellow git, on a near-black #011627 background with teal context line.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/night-owl.toml")),
    },
    Theme {
        name: "material",
        description: "Plain/transparent style with atom-one-dark palette: cyan path, purple model, blue-grey git, vivid green cost — no backgrounds, minimal clutter.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/material.toml")),
    },
    Theme {
        name: "blue-owl",
        description: "Powerline style with deep navy/cobalt blue backgrounds, vivid cyan session text, and dynamic green/yellow/purple git state indicators.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/blue-owl.toml")),
    },
    Theme {
        name: "blueish",
        description: "Powerline style with a cool steel-grey/teal palette: slate-grey default, bright teal path, light-blue git panel, cyan context bar.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/blueish.toml")),
    },
    Theme {
        name: "m365princess",
        description: "Powerline style with a warm feminine palette: blush pink path, plum model, salmon git, sky-blue repo, teal cost — inspired by Microsoft 365 branding.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/m365princess.toml")),
    },
    Theme {
        name: "neko",
        description: "Kawaii plain/transparent theme: warm orange gauge, teal directory, blue git, pink-red repo on a colourless background with zero separators.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/neko.toml")),
    },
    Theme {
        name: "unicorn",
        description: "Powerline theme in dark teal + electric blue + lime-green with a hot-pink unicorn flair; white text on rich coloured segment backgrounds.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/unicorn.toml")),
    },
    Theme {
        name: "multiverse-neon",
        description: "Floating diamond bubbles on dark indigo with neon green directory, electric cyan model, orange repo accent — a sci-fi multiverse aesthetic.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/multiverse-neon.toml")),
    },
    Theme {
        name: "thecyberden",
        description: "Powerline cyber-terminal in electric blue + gold + teal; sharp contrast between blue directory, gold model, teal/dynamic git, and gold cost.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/thecyberden.toml")),
    },
    Theme {
        name: "plague",
        description: "Diamond-cap dark theme: blood-red directory on bg 196, teal context gauge, electric-green cost bar — aggressive high-contrast palette with rounded bubble caps.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/plague.toml")),
    },
    Theme {
        name: "darkblood",
        description: "Plain/transparent style with white text and burnt-orange (#CB4B16 → xterm 166) accents — no backgrounds, minimal bracket-framed aesthetic.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/darkblood.toml")),
    },
    Theme {
        name: "half-life",
        description: "Plain style: electric-green (118) directory, cyan (81) git, purple (97) model, orange (166) cost — no backgrounds, lambda-prompt inspired palette.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/half-life.toml")),
    },
    Theme {
        name: "aliens",
        description: "One Dark palette with rounded-left powerline flow: sky-blue (75) directory, purple (134) model, mint (121) git, coral-pink (204) cost segment.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/aliens.toml")),
    },
    Theme {
        name: "catppuccin-latte",
        description: "Plain/transparent style on a light cream base; Catppuccin Latte palette with blue directory, mauve model, pink git, teal context — dark fg (59) for readability on the bright background.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/catppuccin-latte.toml")),
    },
    Theme {
        name: "catppuccin-macchiato",
        description: "Powerline style on a deep dark-navy base (#24273A→235); Catppuccin Macchiato palette with soft blue directory (111), mauve model (183), lavender context (147), pink cost (218) — pastel accents on dark fills.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/catppuccin-macchiato.toml")),
    },
    Theme {
        name: "catppuccin-frappe",
        description: "Powerline style on a grey-indigo mid-dark base (#303446→237); Catppuccin Frappé palette with soft blue directory (111), mauve model (182), lavender context (147), pink cost (218) — slightly warmer and lighter than Macchiato.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/catppuccin-frappe.toml")),
    },
    Theme {
        name: "tokyo",
        description: "Plain/transparent style matching the OMP Tokyo theme's box-drawing outline aesthetic; steel-blue fg (110) default, white directory (231), purple model (146), pink git (218), orange cost (203) — no filled backgrounds.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/tokyo.toml")),
    },
    Theme {
        name: "tokyonight-storm",
        description: "Plain/transparent style using Tokyo Night Storm's palette: magenta-purple directory, pistachio-green model, sky-blue repo, yellow cost — all on a dark terminal with no background fills.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/tokyonight-storm.toml")),
    },
    Theme {
        name: "the-unnamed",
        description: "Plain/transparent style with bold neon palette: teal directory, hot-pink git, cornflower-blue model, mint repo, canary-yellow cost — high-contrast jewel tones on dark background.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/the-unnamed.toml")),
    },
    Theme {
        name: "space",
        description: "Plain/transparent style with a spacey colour set: light-green directory, violet model, sky-blue git, cyan repo, amber cost — minimal transparent prompt inspired by space theme's open feel.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/space.toml")),
    },
    Theme {
        name: "takuya",
        description: "Diamond style with filled rounded-bubble segments: dark-grey directory, deep-blue model, yellow git, dark-grey repo on line 1; deep-blue context gauge with teal cost — powerline dividers between blocks.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/takuya.toml")),
    },
    Theme {
        name: "tonybaloney",
        description: "Dark navy background (#18354c / xterm 23) with golden amber accents (#ffc107 / xterm 214), diamond separator style with rounded left cap and powerline right arrow — bold two-tone contrast.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/tonybaloney.toml")),
    },
    Theme {
        name: "honukai",
        description: "Plain transparent style with cool blue (#0377C8), forest green (#4A9207), and olive yellow (#B8B80A) — clean minimal text-only look with no backgrounds.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/honukai.toml")),
    },
    Theme {
        name: "di4am0nd",
        description: "Plain transparent, very minimal: gold (#FFBD00) for model, cyan (#00C6F7) for directory, red (#F62F2E) for git — vivid three-colour palette with no backgrounds or separators.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/di4am0nd.toml")),
    },
    Theme {
        name: "smoothie",
        description: "Plain transparent neon palette: hot pink directory (#ffaed8), lime green model (#b1ff4f), sky blue git (#62beff), teal context (#3ce6bf), violet cost (#9966ff) — vivid pastel neon on dark terminal.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/smoothie.toml")),
    },
    Theme {
        name: "sonicboom-dark",
        description: "Pitch-black base with electric cyan (#43CCEA) accents and bright-green git; plain/transparent style — no segment backgrounds, minimal two-colour look.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/sonicboom-dark.toml")),
    },
    Theme {
        name: "wholespace",
        description: "Warm cream/white background (#FEF5ED) with dark navy text, cobalt-blue (#516BEB) model, teal-green (#17D7A0) git and cost; diamond style with standard round caps.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/wholespace.toml")),
    },
    Theme {
        name: "velvet",
        description: "Deep midnight-purple gradient (#0E050F → #69307A) with soft lavender (#EFDCF9) text and lime-yellow model accent; powerline flame divider throughout.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/velvet.toml")),
    },
    Theme {
        name: "ys",
        description: "Terminal-native plain style: no backgrounds, colour-coded text only — light-blue directory, white model, cyan git, yellow context and cost; clean monochrome minimalism.",
        toml: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/ys.toml")),
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
