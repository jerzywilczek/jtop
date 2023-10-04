mod color;
mod theme;

use std::{ffi::OsString, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use color::RgbColor;
pub use theme::*;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Dump the default config
    #[arg(short, long)]
    pub dump_config: bool,

    /// Dump a sample theme
    #[arg(long, short('t'))]
    pub dump_sample_theme: bool,

    /// The path to the config directory
    #[arg(long)]
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub theme: Theme,
}

impl Config {
    pub fn load(cli: &Cli) -> Result<Self> {
        let Some(config_dir_path) = config_path(cli) else {
            return Ok(Default::default());
        };

        let config_file_path = config_dir_path.join("config.toml");

        if !config_file_path.exists() {
            return Ok(Default::default());
        }

        let config = std::fs::read_to_string(&config_file_path).with_context(|| {
            format!(
                "Failed to read config from {}",
                config_file_path.to_string_lossy()
            )
        })?;
        let config = toml::from_str::<RawConfig>(&config).with_context(|| {
            format!(
                "Failed to parse config from {}",
                config_file_path.to_string_lossy()
            )
        })?;

        let Some(theme) = config.theme else {
            return Ok(Self {
                theme: Default::default(),
            });
        };

        let theme = Theme::load_from_file(&config_dir_path.join("themes").join(theme))?;

        Ok(Self { theme })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct RawConfig {
    theme: Option<OsString>,
}

pub fn sample_config() -> String {
    toml::to_string_pretty(&RawConfig {
        theme: Some("theme name (has to correspond with a valid theme file located in <config path>/themes/<theme name>.toml)".into())
    }).unwrap()
}

pub fn config_path(cli: &Cli) -> Option<PathBuf> {
    if let Some(ref path) = cli.config_path {
        if path.exists() {
            return Some(path.clone());
        }
    }

    if let Ok(path) = std::env::var("JWTOP_CONFIG_DIR") {
        let path: PathBuf = path.into();
        if path.exists() {
            return path.into();
        }
    }

    let path = directories::ProjectDirs::from("org", "jw", "jwtop")
        .unwrap()
        .config_dir()
        .to_path_buf();

    if path.exists() {
        return Some(path);
    }

    None
}
