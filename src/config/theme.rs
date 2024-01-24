use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tui::style::Color;

use super::SerdeColor;

mod default_colors {
    use crate::config::SerdeColor;

    pub fn red() -> SerdeColor {
        SerdeColor(tui::style::Color::Red)
    }

    pub fn green() -> SerdeColor {
        SerdeColor(tui::style::Color::Green)
    }

    pub fn yellow() -> SerdeColor {
        SerdeColor(tui::style::Color::Yellow)
    }

    pub fn blue() -> SerdeColor {
        SerdeColor(tui::style::Color::Blue)
    }

    pub fn _magenta() -> SerdeColor {
        SerdeColor(tui::style::Color::Magenta)
    }

    pub fn cyan() -> SerdeColor {
        SerdeColor(tui::style::Color::Cyan)
    }

    pub fn plot() -> Vec<SerdeColor> {
        vec![
            SerdeColor(tui::style::Color::Blue),
            SerdeColor(tui::style::Color::Cyan),
            SerdeColor(tui::style::Color::Green),
            SerdeColor(tui::style::Color::Magenta),
            SerdeColor(tui::style::Color::Red),
            SerdeColor(tui::style::Color::Yellow),
        ]
    }

    pub const RED: SerdeColor = SerdeColor(tui::style::Color::Rgb(0xcc, 0x66, 0x66));
    pub const GREEN: SerdeColor = SerdeColor(tui::style::Color::Rgb(0xb5, 0xbd, 0x68));
    pub const YELLOW: SerdeColor = SerdeColor(tui::style::Color::Rgb(0xf0, 0xc6, 0x74));
    pub const BLUE: SerdeColor = SerdeColor(tui::style::Color::Rgb(0x81, 0xa2, 0xbe));
    pub const MAGENTA: SerdeColor = SerdeColor(tui::style::Color::Rgb(0xb2, 0x94, 0xbb));
    pub const CYAN: SerdeColor = SerdeColor(tui::style::Color::Rgb(0x8a, 0xbe, 0xb7));
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default)]
    pub widget: WidgetTheme,

    #[serde(default)]
    pub plot: PlotTheme,

    #[serde(default)]
    pub bars: BarsTheme,

    #[serde(default)]
    pub table: TableTheme,
}

impl Theme {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme from {}", path.to_string_lossy()))?;

        let theme: Theme = toml::from_str(&file).context("Failed to deserialize theme")?;

        Ok(theme)
    }

    pub fn sample_theme() -> Self {
        Self {
            widget: WidgetTheme {
                frame_color: default_colors::CYAN,
                title_color: default_colors::CYAN,
                background_color: Some(SerdeColor(tui::style::Color::Rgb(0x00, 0x2b, 0x36))),
            },

            plot: PlotTheme {
                axis_labels_color: default_colors::CYAN,
                plot_colors: vec![
                    default_colors::BLUE,
                    default_colors::CYAN,
                    default_colors::GREEN,
                    default_colors::MAGENTA,
                    default_colors::RED,
                    default_colors::YELLOW,
                ],
            },

            bars: BarsTheme {
                low_usage_color: default_colors::GREEN,
                medium_usage_color: default_colors::YELLOW,
                high_usage_color: default_colors::RED,
            },

            table: TableTheme {
                header_color: default_colors::BLUE,
                row_color: default_colors::BLUE,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WidgetTheme {
    #[serde(default = "default_colors::cyan")]
    pub frame_color: SerdeColor,
    #[serde(default = "default_colors::cyan")]
    pub title_color: SerdeColor,
    #[serde(default)]
    pub background_color: Option<SerdeColor>,
}

impl Default for WidgetTheme {
    fn default() -> Self {
        Self {
            frame_color: SerdeColor(Color::Cyan),
            title_color: SerdeColor(Color::Cyan),
            background_color: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotTheme {
    #[serde(default = "default_colors::cyan")]
    pub axis_labels_color: SerdeColor,
    #[serde(default = "default_colors::plot")]
    pub plot_colors: Vec<SerdeColor>,
}

impl Default for PlotTheme {
    fn default() -> Self {
        Self {
            axis_labels_color: SerdeColor(Color::Cyan),
            plot_colors: default_colors::plot(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BarsTheme {
    #[serde(default = "default_colors::green")]
    pub low_usage_color: SerdeColor,
    #[serde(default = "default_colors::yellow")]
    pub medium_usage_color: SerdeColor,
    #[serde(default = "default_colors::red")]
    pub high_usage_color: SerdeColor,
}

impl Default for BarsTheme {
    fn default() -> Self {
        Self {
            low_usage_color: SerdeColor(Color::Green),
            medium_usage_color: SerdeColor(Color::Yellow),
            high_usage_color: SerdeColor(Color::Red),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TableTheme {
    #[serde(default = "default_colors::blue")]
    pub header_color: SerdeColor,
    #[serde(default = "default_colors::blue")]
    pub row_color: SerdeColor,
}

impl Default for TableTheme {
    fn default() -> Self {
        Self {
            header_color: SerdeColor(Color::Blue),
            row_color: SerdeColor(Color::Blue),
        }
    }
}
