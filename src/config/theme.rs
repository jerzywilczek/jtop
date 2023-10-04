use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tui::style::Color;

use super::RgbColor;

// TODO: actually, this is kind of stupid. Most of this can be handled by using serde attributes.

mod default_colors {
    use crate::config::RgbColor;

    pub const RED: RgbColor = RgbColor(0xcc, 0x66, 0x66);
    pub const GREEN: RgbColor = RgbColor(0xb5, 0xbd, 0x68);

    pub const YELLOW: RgbColor = RgbColor(0xf0, 0xc6, 0x74);

    pub const BLUE: RgbColor = RgbColor(0x81, 0xa2, 0xbe);

    pub const MAGENTA: RgbColor = RgbColor(0xb2, 0x94, 0xbb);

    pub const CYAN: RgbColor = RgbColor(0x8a, 0xbe, 0xb7);

    pub const BG: RgbColor = RgbColor(0x1d, 0x1f, 0x21);
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RawTheme {
    widget: Option<RawWidgetTheme>,
    plot: Option<RawPlotTheme>,
    bars: Option<RawBarsTheme>,
    table: Option<RawTableTheme>,
}

impl RawTheme {
    fn sample_theme() -> Self {
        Self {
            widget: Some(RawWidgetTheme {
                frame_color: Some(default_colors::CYAN),
                title_color: Some(default_colors::CYAN),
                background_color: Some(default_colors::BG),
            }),

            plot: Some(RawPlotTheme {
                axis_labels_color: Some(default_colors::CYAN),
                plot_colors: vec![
                    default_colors::BLUE,
                    default_colors::CYAN,
                    default_colors::GREEN,
                    default_colors::MAGENTA,
                    default_colors::RED,
                    default_colors::YELLOW,
                ],
            }),

            bars: Some(RawBarsTheme {
                low_usage_color: Some(default_colors::GREEN),
                medium_usage_color: Some(default_colors::YELLOW),
                high_usage_color: Some(default_colors::RED),
            }),

            table: Some(RawTableTheme {
                header_color: Some(default_colors::BLUE),
                row_color: Some(default_colors::BLUE),
            }),
        }
    }
}

pub fn sample_theme() -> String {
    toml::to_string_pretty(&RawTheme::sample_theme()).unwrap()
}

fn tui_color(raw: Option<RgbColor>, default: Color) -> Color {
    raw.map(|c| c.into()).unwrap_or(default)
}

#[derive(Debug, Default, Clone)]
pub struct Theme {
    pub widget: WidgetTheme,
    pub plot: PlotTheme,
    pub bars: BarsTheme,
    pub table: TableTheme,
}

impl From<RawTheme> for Theme {
    fn from(value: RawTheme) -> Self {
        Self {
            widget: value.widget.map(|t| t.into()).unwrap_or_default(),
            plot: value.plot.map(|t| t.into()).unwrap_or_default(),
            bars: value.bars.map(|t| t.into()).unwrap_or_default(),
            table: value.table.map(|t| t.into()).unwrap_or_default(),
        }
    }
}

impl Theme {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme from {}", path.to_string_lossy()))?;

        let raw: RawTheme = toml::from_str(&file).context("Failed to deserialize theme")?;

        Ok(raw.into())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct RawWidgetTheme {
    frame_color: Option<RgbColor>,
    title_color: Option<RgbColor>,
    background_color: Option<RgbColor>,
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetTheme {
    pub frame_color: Color,
    pub title_color: Color,
}

impl Default for WidgetTheme {
    fn default() -> Self {
        Self {
            frame_color: Color::Cyan,
            title_color: Color::Cyan,
        }
    }
}

impl From<RawWidgetTheme> for WidgetTheme {
    fn from(value: RawWidgetTheme) -> Self {
        let default = Self::default();

        Self {
            frame_color: tui_color(value.frame_color, default.frame_color),
            title_color: tui_color(value.title_color, default.title_color),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RawPlotTheme {
    axis_labels_color: Option<RgbColor>,
    plot_colors: Vec<RgbColor>,
}

#[derive(Debug, Clone)]
pub struct PlotTheme {
    pub axis_labels_color: Color,
    pub plot_colors: Vec<Color>,
}

impl Default for PlotTheme {
    fn default() -> Self {
        Self {
            axis_labels_color: Color::Cyan,
            plot_colors: vec![
                Color::Blue,
                Color::Cyan,
                Color::Green,
                Color::Magenta,
                Color::Red,
                Color::Yellow,
            ],
        }
    }
}

impl From<RawPlotTheme> for PlotTheme {
    fn from(value: RawPlotTheme) -> Self {
        let default = Self::default();

        Self {
            axis_labels_color: tui_color(value.axis_labels_color, default.axis_labels_color),
            plot_colors: if !value.plot_colors.is_empty() {
                value
                    .plot_colors
                    .into_iter()
                    .map(|c| c.into())
                    .collect::<Vec<_>>()
            } else {
                default.plot_colors
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct RawBarsTheme {
    low_usage_color: Option<RgbColor>,
    medium_usage_color: Option<RgbColor>,
    high_usage_color: Option<RgbColor>,
}

#[derive(Debug, Clone, Copy)]
pub struct BarsTheme {
    pub low_usage_color: Color,
    pub medium_usage_color: Color,
    pub high_usage_color: Color,
}

impl Default for BarsTheme {
    fn default() -> Self {
        Self {
            low_usage_color: Color::Green,
            medium_usage_color: Color::Yellow,
            high_usage_color: Color::Red,
        }
    }
}

impl From<RawBarsTheme> for BarsTheme {
    fn from(value: RawBarsTheme) -> Self {
        let default = Self::default();

        Self {
            low_usage_color: tui_color(value.low_usage_color, default.low_usage_color),
            medium_usage_color: tui_color(value.medium_usage_color, default.medium_usage_color),
            high_usage_color: tui_color(value.high_usage_color, default.high_usage_color),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct RawTableTheme {
    header_color: Option<RgbColor>,
    row_color: Option<RgbColor>,
}

#[derive(Debug, Clone, Copy)]
pub struct TableTheme {
    pub header_color: Color,
    pub row_color: Color,
}

impl Default for TableTheme {
    fn default() -> Self {
        Self {
            header_color: Color::Blue,
            row_color: Color::Blue,
        }
    }
}

impl From<RawTableTheme> for TableTheme {
    fn from(value: RawTableTheme) -> Self {
        let default = Self::default();

        Self {
            header_color: tui_color(value.header_color, default.header_color),
            row_color: tui_color(value.row_color, default.row_color),
        }
    }
}
