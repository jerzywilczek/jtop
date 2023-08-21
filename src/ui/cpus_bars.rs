use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Widget},
};

use crate::app::App;

pub struct CpusBars<'a> {
    cpus: Vec<f64>,
    style: Style,
    block: Option<Block<'a>>,
}

impl<'a> CpusBars<'a> {
    pub fn new(app: &App) -> Self {
        let cpus = app
            .cpu_history
            .iter()
            .flat_map(|v| v.back())
            .copied()
            .collect();

        Self {
            cpus,
            style: Default::default(),
            block: Default::default(),
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn block(self, block: Block) -> CpusBars {
        CpusBars {
            block: Some(block),
            ..self
        }
    }
}

impl<'a> Widget for CpusBars<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let vertical_amount = (self.cpus.len() as u32 + 3) / 4;

        buf.set_style(area, self.style);

        let area = match self.block.clone() {
            Some(block) => {
                let inner = block.inner(area);
                block.render(area, buf);
                inner
            }
            None => area,
        };

        let rows = Layout::default()
            .margin(0)
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Ratio(1, vertical_amount);
                vertical_amount as usize
            ])
            .split(area);

        let cells = rows
            .iter()
            .map(|&row| {
                Layout::default()
                    .margin(0)
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Ratio(1, 4); 4])
                    .split(row)
            })
            .collect::<Vec<_>>();

        cells
            .iter()
            .flat_map(|c| c.iter())
            .copied()
            .zip(self.cpus)
            .enumerate()
            .for_each(|(i, (area, val))| {
                let color = if val < 50.0 {
                    Color::Green
                } else if val < 80.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };

                Gauge::default()
                    .label(format!("cpu{i}: {val:.2}%"))
                    .gauge_style(Style::default().fg(color))
                    .ratio(val / 100.0)
                    .block(Block::default().borders(Borders::empty()).padding(
                        tui::widgets::Padding {
                            left: 1,
                            right: if i % 4 == 3 { 1 } else { 0 },
                            top: 0,
                            bottom: 0,
                        },
                    ))
                    .render(area, buf);
            });
    }
}
