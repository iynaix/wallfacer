use super::aspect_ratio::AspectRatio;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("No config file found")]
    NotFound,
    #[error("Invalid config")]
    InvalidConfig,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigResolution {
    pub name: String,
    pub description: Option<String>,
    pub resolution: AspectRatio,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub min_width: u32,
    pub min_height: u32,
    pub show_faces: bool,
    pub resolutions: Vec<ConfigResolution>,
    pub wallpaper_command: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_width: 1920,
            min_height: 1080,
            show_faces: false,
            resolutions: vec![ConfigResolution {
                name: "HD".into(),
                description: Some("Full HD (1920x1080)".into()),
                resolution: AspectRatio::new(1920, 1080),
            }],
            wallpaper_command: None,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let cfg_file = dirs::config_dir()
            .expect("could not get xdg config directory")
            .join("wallfacer/wallfacer.toml");

        let contents = std::fs::read_to_string(cfg_file).map_err(|_| ConfigError::NotFound)?;
        let mut cfg: Self = toml::from_str(&contents).map_err(|_| ConfigError::InvalidConfig)?;
        cfg.resolutions.sort_by_key(|res| res.resolution.clone());
        Ok(cfg)
    }

    pub fn sorted_resolutions(&self) -> Vec<AspectRatio> {
        self.resolutions
            .iter()
            .map(|res| res.resolution.clone())
            .collect()
    }

    /// saves the current configuration
    pub fn save(&self) -> std::io::Result<()> {
        let cfg_file = dirs::config_dir()
            .expect("could not get xdg config directory")
            .join("wallfacer/wallfacer.toml");

        let toml = toml::to_string(self).expect("could not serialize config");

        std::fs::write(cfg_file, toml)
    }
}
