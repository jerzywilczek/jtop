use tui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Row, Table, Widget},
};

use crate::app::{App, ProcessInfo};

pub struct Processes<'b> {
    processes: Vec<ProcessInfo>,
    style: Style,
    block: Option<Block<'b>>,
}

impl<'b> Processes<'b> {
    pub fn new(app: &App) -> Self {
        Self {
            processes: app.processes.clone(),
            style: Default::default(),
            block: Default::default(),
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn block<'c>(self, block: Block<'c>) -> Processes<'c> {
        Processes {
            block: Some(block),
            ..self
        }
    }
}

fn mem_string(mem: u64) -> String {
    const PREFIXES: &[&str] = &["B", "K", "M", "G", "T", "P"];

    let mut mem = mem as f64;

    for &prefix in PREFIXES {
        if mem <= 1_000.0 {
            return format!("{mem:.1}{prefix}");
        }

        mem /= 1_000.0;
    }

    format!("{mem:.1}{}", PREFIXES.last().unwrap())
}

impl<'b> Widget for Processes<'b> {
    fn render(mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        self.processes.sort_by(|p1, p2| p2.cpu.total_cmp(&p1.cpu));

        Table::new(self.processes.into_iter().map(|p| {
            Row::new([
                p.pid.to_string(),
                p.name,
                format!("{:.01}%", p.cpu),
                mem_string(p.mem),
                mem_string(p.disk_r),
                mem_string(p.disk_w),
            ])
            .style(Style::default().fg(tui::style::Color::Blue))
        }))
        .column_spacing(1)
        .widths(&[Constraint::Ratio(1, 6); 6])
        .block(self.block.unwrap_or_default())
        .style(self.style)
        .header(
            Row::new(["pid", "name", "cpu", "mem", "disk r/s", "disk w/s"]).style(
                Style::default()
                    .fg(tui::style::Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .render(area, buf);
    }
}
