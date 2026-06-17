pub mod rules;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::browser::Browser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_repositories() -> Vec<Repository> {
    vec![Repository {
        name: "Official".to_string(),
        url: crate::plugin::CHANNEL_URL.to_string(),
        enabled: true,
    }]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_browser: Option<Browser>,
    #[serde(default)]
    pub always_show_selector: bool,
    #[serde(default)]
    pub rules: Vec<rules::Rule>,
    #[serde(default)]
    pub remembered: Vec<Remembered>,
    #[serde(default)]
    pub hotkeys: HashMap<String, String>,
    #[serde(default)]
    pub installed_plugins: Vec<crate::plugin::InstalledPlugin>,
    #[serde(default = "default_repositories")]
    pub repositories: Vec<Repository>,
    /// Manually added browsers (name + .app path).
    #[serde(default)]
    pub custom_browsers: Vec<Browser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remembered {
    pub domain: String,
    pub browser: Browser,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_browser: Some(Browser::Chrome { app_path: None }),
            always_show_selector: false,
            rules: vec![],
            remembered: vec![],
            hotkeys: HashMap::new(),
            installed_plugins: vec![],
            custom_browsers: vec![],
            repositories: default_repositories(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {:?}", path))?;
        let config: Config =
            serde_yaml::from_str(&content).context("failed to parse config YAML")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("failed to create config directory")?;
        }
        let content = serde_yaml::to_string(self).context("failed to serialize config")?;
        std::fs::write(&path, content)
            .with_context(|| format!("failed to write config to {:?}", path))?;
        Ok(())
    }
}

fn config_path() -> Result<PathBuf> {
    let proj = directories::ProjectDirs::from("com", "browseraptor", "browseraptor")
        .context("failed to determine config directory")?;
    Ok(proj.config_dir().join("config.yaml"))
}
