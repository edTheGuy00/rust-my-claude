use serde::Deserialize;
use std::io::{self, Read};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── JSON input schema ────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct Input {
    #[serde(default)]
    model: Model,
    #[serde(default)]
    workspace: Workspace,
    #[serde(default)]
    cost: Cost,
    #[serde(default)]
    context_window: ContextWindow,
    #[serde(default)]
    rate_limits: Option<RateLimits>,
    #[serde(default)]
    vim: Option<VimState>,
    #[serde(default)]
    thinking: Option<Thinking>,
}

#[derive(Deserialize, Default)]
struct Model {
    #[serde(default)]
    display_name: String,
}

#[derive(Deserialize, Default)]
struct Workspace {
    #[serde(default)]
    current_dir: String,
    #[serde(default)]
    repo: Option<Repo>,
}

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct Repo {
    #[serde(default)]
    owner: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Default)]
struct Cost {
    #[serde(default)]
    total_cost_usd: f64,
    #[serde(default)]
    total_duration_ms: u64,
}

#[derive(Deserialize, Default)]
struct ContextWindow {
    #[serde(default)]
    used_percentage: Option<f64>,
    #[serde(default)]
    context_window_size: u64,
    #[serde(default)]
    total_input_tokens: u64,
}

#[derive(Deserialize, Default)]
struct RateLimits {
    five_hour: Option<RateWindow>,
    seven_day: Option<RateWindow>,
}

#[derive(Deserialize, Default)]
struct RateWindow {
    used_percentage: Option<f64>,
    resets_at: Option<u64>,
}

#[derive(Deserialize, Default)]
struct VimState {
    #[serde(default)]
    mode: String,
}

#[derive(Deserialize, Default)]
struct Thinking {
    #[serde(default)]
    enabled: bool,
}

// ─── Powerline rendering ────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const SEP:   char = '\u{e0b0}';   //  hard divider
const CAP_L: char = '\u{e0b6}';   //  rounded left cap
const CAP_R: char = '\u{e0b4}';   //  rounded right cap

// Nerd Font segment icons
const I_DIR:   &str = "\u{f07b}";  //  folder
const I_GIT:   &str = "\u{e0a0}";  //  branch
const I_MODEL: &str = "\u{2733}";  // ✳ Claude sunburst
const I_CTX:   &str = "\u{f0e4}";  //  gauge
const I_COST:  &str = "\u{f155}";  //  dollar
const I_FIRE:  &str = "\u{f06d}";  //  fire (5h window)
const I_CAL:   &str = "\u{f073}";  //  calendar (7d window)
const I_VIM:   &str = "\u{e62b}";  //  vim
const I_THINK: &str = "\u{f0eb}";  //  lightbulb

// Segment background colours (256-colour indices)
const BG_DIR:    u8 = 208;   // orange
const BG_MODEL:  u8 = 24;    // navy
const BG_REPO:   u8 = 238;   // dark grey
const BG_VIM:    u8 = 30;    // teal
const BG_THINK:  u8 = 168;   // pink
const BG_CLEAN:  u8 = 71;    // green  – clean tree
const BG_STAGED: u8 = 35;    // emerald – staged changes
const BG_DIRTY:  u8 = 178;   // gold   – unstaged / untracked
const BG_COST:   u8 = 94;    // brown

// Foreground (text) colours: dark for light pills, light for dark pills
const FG_DARK:  u8 = 235;
const FG_LIGHT: u8 = 231;
const FG_REPO:  u8 = 250;

// Gauge pill colours: (filled, track). Each metric gets its own hue so the pills
// are distinguishable; within a pill the filled fraction shows the percentage and
// the remainder is a darker shade of the same hue. The filled hue shifts toward
// red as usage climbs past ~60% (see `shift_to_red`).
const G_CTX: (u8, u8) = (99,  54);   // violet  – context window
const G_5H:  (u8, u8) = (168, 89);   // magenta – 5-hour window
const G_7D:  (u8, u8) = (33,  24);   // blue    – 7-day window

// ─── Colour interpolation (for the red-shift on gauge pills) ────────────────────

/// xterm-256 palette index → RGB.
fn xterm_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0..=15 => {
            const BASE: [(u8, u8, u8); 16] = [
                (0,0,0),(205,0,0),(0,205,0),(205,205,0),(0,0,238),(205,0,205),
                (0,205,205),(229,229,229),(127,127,127),(255,0,0),(0,255,0),
                (255,255,0),(92,92,255),(255,0,255),(0,255,255),(255,255,255),
            ];
            BASE[n as usize]
        }
        16..=231 => {
            let n = n - 16;
            let lvl = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            (lvl(n / 36), lvl((n % 36) / 6), lvl(n % 6))
        }
        _ => { let v = 8 + (n - 232) * 10; (v, v, v) }
    }
}

/// RGB → nearest index in the 6×6×6 xterm colour cube.
fn nearest_xterm(r: u8, g: u8, b: u8) -> u8 {
    const LV: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let idx = |v: u8| {
        let (mut best, mut bd) = (0u8, u16::MAX);
        for (i, &l) in LV.iter().enumerate() {
            let d = (l as i16 - v as i16).unsigned_abs();
            if d < bd { bd = d; best = i as u8; }
        }
        best
    };
    16 + 36 * idx(r) + 6 * idx(g) + idx(b)
}

/// Blend `base` toward red as `pct` rises: unchanged below 60%, fully red at 100%.
fn shift_to_red(base: u8, pct: f64) -> u8 {
    let t = ((pct - 60.0) / 40.0).clamp(0.0, 1.0);
    if t == 0.0 { return base; }
    let (br, bg, bb) = xterm_rgb(base);
    let lerp = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
    nearest_xterm(lerp(br, 255), lerp(bg, 0), lerp(bb, 0))
}

/// One Powerline segment: an icon + text on a colour block. When `fill` is set
/// the block doubles as a progress bar — cells up to `pct` use `bg`, the rest
/// use the darker track colour.
struct Pill {
    icon: &'static str,
    text: String,
    fg:   u8,
    bg:   u8,
    fill: Option<(u8, f64)>,   // (track colour, percent filled)
}

impl Pill {
    /// A solid-colour pill.
    fn new(icon: &'static str, text: impl Into<String>, fg: u8, bg: u8) -> Self {
        Pill { icon, text: text.into(), fg, bg, fill: None }
    }

    /// A pill whose background fills left-to-right in proportion to `pct`, using
    /// the given `(filled, track)` colour pair. The filled hue bleeds toward red
    /// as `pct` approaches 100%; the track keeps the pill's identity hue.
    fn gauge(icon: &'static str, text: impl Into<String>, pct: f64, colours: (u8, u8)) -> Self {
        let (filled, track) = colours;
        Pill { icon, text: text.into(), fg: FG_LIGHT, bg: shift_to_red(filled, pct), fill: Some((track, pct)) }
    }

    /// Render the body and report the (leading, trailing) cell background colours
    /// so caps and dividers can match the visible edges.
    fn body(&self) -> (u8, String, u8) {
        let label = if self.icon.is_empty() {
            format!(" {} ", self.text)
        } else {
            format!(" {} {} ", self.icon, self.text)
        };
        match self.fill {
            None => (self.bg, format!("\x1b[48;5;{};38;5;{}m{label}", self.bg, self.fg), self.bg),
            Some((track, pct)) => {
                let chars: Vec<char> = label.chars().collect();
                let total = chars.len().max(1);
                let filled = (((pct / 100.0) * total as f64).round() as usize).min(total);
                let mut s = String::new();
                let mut cur: i16 = -1;
                for (i, c) in chars.iter().enumerate() {
                    let bg = if i < filled { self.bg } else { track };
                    if bg as i16 != cur {
                        s.push_str(&format!("\x1b[48;5;{};38;5;{}m", bg, self.fg));
                        cur = bg as i16;
                    }
                    s.push(*c);
                }
                let leading  = if filled >= 1     { self.bg } else { track };
                let trailing = if filled >= total { self.bg } else { track };
                (leading, s, trailing)
            }
        }
    }
}

/// Render a chain of pills into one line: rounded caps at the ends, `` between.
fn render(pills: &[Pill]) -> String {
    if pills.is_empty() { return String::new(); }
    let bodies: Vec<(u8, String, u8)> = pills.iter().map(Pill::body).collect();
    let mut out = format!("\x1b[38;5;{}m{}", bodies[0].0, CAP_L);
    for (i, (_, body, trailing)) in bodies.iter().enumerate() {
        out.push_str(body);
        match bodies.get(i + 1) {
            // divider: next pill's leading bg behind, this pill's trailing bg as the arrow
            Some((next_lead, _, _)) => out.push_str(&format!("\x1b[48;5;{};38;5;{}m{}", next_lead, trailing, SEP)),
            // right cap: drop back to the terminal bg, draw the cap in this pill's trailing colour
            None => out.push_str(&format!("{RESET}\x1b[38;5;{}m{CAP_R}{RESET}", trailing)),
        }
    }
    out
}

// ─── Git ──────────────────────────────────────────────────────────────────────

struct GitInfo {
    branch:    String,
    staged:    usize,
    modified:  usize,
    untracked: usize,
    ahead:     usize,
    behind:    usize,
}

fn git_info(cwd: &str) -> Option<GitInfo> {
    let in_repo = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_repo { return None; }

    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let status_out = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .current_dir(cwd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let (mut staged, mut modified, mut untracked) = (0usize, 0usize, 0usize);
    for line in status_out.lines() {
        if line.len() < 2 { continue; }
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        if x == '?' && y == '?' { untracked += 1; }
        else {
            if x != ' ' && x != '?' { staged   += 1; }
            if y != ' ' && y != '?' { modified += 1; }
        }
    }

    let (ahead, behind) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "@{u}...HEAD"])
        .current_dir(cwd)
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            let v: Vec<&str> = s.split_whitespace().collect();
            if v.len() == 2 {
                (v[1].parse().unwrap_or(0), v[0].parse().unwrap_or(0))
            } else { (0, 0) }
        })
        .unwrap_or((0, 0));

    Some(GitInfo { branch, staged, modified, untracked, ahead, behind })
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn fmt_duration(ms: u64) -> String {
    let s = ms / 1000;
    let m = s / 60;
    if m > 0 { format!("{m}m{:02}s", s % 60) } else { format!("{s}s") }
}

fn fmt_countdown(resets_at: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if resets_at <= now { return "now".into(); }
    let d = resets_at - now;
    let h = d / 3600;
    let m = (d % 3600) / 60;
    if h > 0 { format!("{h}h{m:02}m") } else { format!("{m}m") }
}

/// Format an epoch-seconds reset point as a local calendar date, e.g. "Jun 19".
/// Shells out to `date` so the local timezone is respected; falls back to the
/// countdown form if `date` is unavailable or fails.
fn fmt_date(resets_at: u64) -> String {
    Command::new("date")
        .args(["-d", &format!("@{resets_at}"), "+%b %-d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fmt_countdown(resets_at))
}

fn short_dir(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let p = if !home.is_empty() && path.starts_with(&home) {
        path.replacen(&home, "~", 1)
    } else { path.to_string() };
    let parts: Vec<&str> = p.trim_end_matches('/').split('/').collect();
    let n = parts.len();
    if n <= 2 { p } else { format!("{}/{}", parts[n-2], parts[n-1]) }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).ok();

    let input: Input = serde_json::from_str(&raw).unwrap_or_default();

    let cwd        = &input.workspace.current_dir;
    let model      = &input.model.display_name;
    let cost       = input.cost.total_cost_usd;
    let dur_ms     = input.cost.total_duration_ms;
    let ctx_pct    = input.context_window.used_percentage.unwrap_or(0.0);
    let ctx_size   = input.context_window.context_window_size;
    let ctx_tokens = input.context_window.total_input_tokens;
    let thinking   = input.thinking.as_ref().map(|t| t.enabled).unwrap_or(false);
    let vim_mode   = input.vim.as_ref().map(|v| v.mode.as_str()).unwrap_or("");

    // ── LINE 1 ── dir · model · git [· repo] [· vim] [· thinking] ─────────────

    let mut line1: Vec<Pill> = Vec::new();
    line1.push(Pill::new(I_DIR, short_dir(cwd), FG_DARK, BG_DIR));

    if !model.is_empty() {
        line1.push(Pill::new(I_MODEL, model.clone(), FG_LIGHT, BG_MODEL));
    }

    if !cwd.is_empty() && let Some(g) = git_info(cwd) {
        let bg = if g.staged > 0 { BG_STAGED }
                 else if g.modified > 0 || g.untracked > 0 { BG_DIRTY }
                 else { BG_CLEAN };

        let mut t = if g.branch.is_empty() { "(detached)".to_string() } else { g.branch.clone() };
        if g.staged    > 0 { t.push_str(&format!(" +{}", g.staged)); }
        if g.modified  > 0 { t.push_str(&format!(" ~{}", g.modified)); }
        if g.untracked > 0 { t.push_str(&format!(" ?{}", g.untracked)); }
        if g.ahead     > 0 { t.push_str(&format!(" ↑{}", g.ahead)); }
        if g.behind    > 0 { t.push_str(&format!(" ↓{}", g.behind)); }
        line1.push(Pill::new(I_GIT, t, FG_DARK, bg));
    }

    if let Some(r) = &input.workspace.repo && !r.name.is_empty() {
        line1.push(Pill::new("", format!("{}/{}", r.owner, r.name), FG_REPO, BG_REPO));
    }

    if !vim_mode.is_empty() && vim_mode != "INSERT" {
        line1.push(Pill::new(I_VIM, vim_mode, FG_LIGHT, BG_VIM));
    }

    if thinking {
        line1.push(Pill::new(I_THINK, "thinking", FG_DARK, BG_THINK));
    }

    // ── LINE 2 ── context · 5h limit · 7d limit · cost ────────────────────────
    // The context / 5h / 7d pills are gauges: background fills with usage %.

    let mut line2: Vec<Pill> = Vec::new();

    let ctx_text = if ctx_tokens > 0 && ctx_size > 0 {
        format!("{:.0}% · {}k/{}k", ctx_pct, ctx_tokens / 1000, ctx_size / 1000)
    } else {
        format!("{ctx_pct:.0}%")
    };
    line2.push(Pill::gauge(I_CTX, ctx_text, ctx_pct, G_CTX));

    if let Some(rl) = &input.rate_limits {
        // 5h window: usage % + countdown to reset
        if let Some(fh) = &rl.five_hour {
            let pct = fh.used_percentage.unwrap_or(0.0);
            let mut t = format!("5h {pct:.0}%");
            if let Some(at) = fh.resets_at { t.push_str(&format!(" ↺{}", fmt_countdown(at))); }
            line2.push(Pill::gauge(I_FIRE, t, pct, G_5H));
        }
        // 7d window: usage % + reset date (e.g. "Jun 19")
        if let Some(sd) = &rl.seven_day {
            let pct = sd.used_percentage.unwrap_or(0.0);
            let mut t = format!("7d {pct:.0}%");
            if let Some(at) = sd.resets_at { t.push_str(&format!(" {}", fmt_date(at))); }
            line2.push(Pill::gauge(I_CAL, t, pct, G_7D));
        }
    }

    if cost > 0.0 || dur_ms > 0 {
        line2.push(Pill::new(I_COST, format!("{:.4} · {}", cost, fmt_duration(dur_ms)), FG_LIGHT, BG_COST));
    }

    println!("{}", render(&line1));
    println!("{}", render(&line2));
}
