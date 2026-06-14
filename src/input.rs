//! Serde structs for the Claude Code statusline JSON fed on stdin.
//!
//! All fields are `#[serde(default)]` so partial JSON (or empty input) produces
//! a zero-value `Input` rather than a deserialization error.
//!
//! Reference: https://code.claude.com/docs/en/statusline

use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
pub struct Input {
    #[serde(default)]
    pub model: Model,
    #[serde(default)]
    pub workspace: Workspace,
    #[serde(default)]
    pub cost: Cost,
    #[serde(default)]
    pub context_window: ContextWindow,
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
    #[serde(default)]
    pub vim: Option<VimState>,
    #[serde(default)]
    pub thinking: Option<Thinking>,
    #[serde(default)]
    pub effort: Option<Effort>,
    #[serde(default)]
    pub pr: Option<Pr>,
    #[serde(default)]
    pub session_name: Option<String>,
    #[serde(default)]
    pub output_style: Option<OutputStyle>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct Model {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub id: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct Workspace {
    #[serde(default)]
    pub current_dir: String,
    #[serde(default)]
    pub project_dir: Option<String>,
    #[serde(default)]
    pub repo: Option<Repo>,
}

#[derive(Deserialize, Default, Clone)]
pub struct Repo {
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct Cost {
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_duration_ms: u64,
    #[serde(default)]
    pub total_lines_added: u64,
    #[serde(default)]
    pub total_lines_removed: u64,
}

#[derive(Deserialize, Default, Clone)]
pub struct ContextWindow {
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub context_window_size: u64,
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub total_output_tokens: u64,
}

#[derive(Deserialize, Default, Clone)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Deserialize, Default, Clone)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<u64>,
}

#[derive(Deserialize, Default, Clone)]
pub struct VimState {
    #[serde(default)]
    pub mode: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct Thinking {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Deserialize, Default, Clone)]
pub struct Effort {
    #[serde(default)]
    pub level: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct Pr {
    pub number: Option<u64>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub review_state: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct OutputStyle {
    #[serde(default)]
    pub name: String,
}
