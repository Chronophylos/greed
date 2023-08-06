use std::time::Duration;

use miette::{IntoDiagnostic, Result};
use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "greed.toml";
const DEFAULT_NTFY_SERVER: &str = "https://ntfy.sh";
const DEFAULT_ITERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour

fn default_user_agent() -> String {
    format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    pub selenium: SeleniumConfig,
    pub telegram: TelegramConfig,
    pub ntfy: NtfyConfig,
    pub sites: Vec<SiteConfig>,
}

fn default_interval() -> Duration {
    DEFAULT_ITERVAL
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub name: String,
    pub url: String,
    #[serde(default = "default_interval", with = "humantime_serde")]
    pub interval: Duration,
    #[serde(default)]
    pub use_browser: bool,
    pub selector: String,
    #[serde(default)]
    pub transformers: Vec<Transformer>,
    pub rules: Vec<RuleConfig>,
    pub notifiers: Vec<Notifiers>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Notifiers {
    //Telegram,
    //Twitch,
    //Email,
    Ntfy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RuleConfig {
    OnChange,
    OnChangeFrom(String),
    OnChangeTo(String),
    OnChangeFromTo(String, String),
    OnDecrease,
    OnIncrease,
    LessThan(f64),
    LessThanOrEqualTo(f64),
    EqualTo(f64),
    MoreThan(f64),
    MoreThanOrEqualTo(f64),
}

#[derive(Debug, Clone, Deserialize)]
pub enum Transformer {
    RegexExtract(String),
    Replace { from: String, to: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeleniumConfig {
    pub url: String,
    pub driver: SeleniumDriver,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeleniumDriver {
    Chrome,
    Firefox,
}

fn default_ntfy_server() -> String {
    DEFAULT_NTFY_SERVER.to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct NtfyConfig {
    #[serde(default = "default_ntfy_server")]
    pub server: String,
    pub topic: String,
}

pub fn read_config() -> Result<Config> {
    let data = std::fs::read_to_string(DEFAULT_CONFIG_PATH).into_diagnostic()?;
    let config = toml::from_str(&data).into_diagnostic()?;

    Ok(config)
}
