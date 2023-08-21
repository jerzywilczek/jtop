use std::{cmp::Ordering, fmt::Display};

use tui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Row, Table, Widget},
};

use crate::app::{App, ProcessInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn reversed(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Pid,
    Name,
    Cpu,
    Memory,
    DiskRead,
    DiskWrite,
}

impl Column {
    pub const ALL_COLUMNS: &[Column] = &[
        Column::Pid,
        Column::Name,
        Column::Cpu,
        Column::Memory,
        Column::DiskRead,
        Column::DiskWrite,
    ];

    // TODO: add comparators here and sort

    fn extract_data_as_string(&self, info: &ProcessInfo) -> String {
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

        match self {
            Column::Pid => info.pid.to_string(),
            Column::Name => info.name.clone(),
            Column::Cpu => format!("{:.01}%", info.cpu),
            Column::Memory => mem_string(info.mem),
            Column::DiskRead => mem_string(info.disk_r),
            Column::DiskWrite => mem_string(info.disk_w),
        }
    }

    fn compare_by(&self, p1: &ProcessInfo, p2: &ProcessInfo) -> Ordering {
        match self {
            Column::Pid => p1.pid.cmp(&p2.pid),
            Column::Name => p1.name.to_lowercase().cmp(&p2.name),
            Column::Cpu => p1.cpu.total_cmp(&p2.cpu),
            Column::Memory => p1.mem.cmp(&p2.mem),
            Column::DiskRead => p1.disk_r.cmp(&p2.disk_r),
            Column::DiskWrite => p1.disk_w.cmp(&p2.disk_w),
        }
    }

    pub fn default_sort_direction(&self) -> SortDirection {
        match self {
            Column::Pid | Column::Name => SortDirection::Ascending,

            Column::Cpu | Column::Memory | Column::DiskRead | Column::DiskWrite => {
                SortDirection::Descending
            }
        }
    }

    fn sort_arrow_str(&self, sorted_column: Column, sort_direction: SortDirection) -> &str {
        if self != &sorted_column {
            return "";
        }

        if sort_direction == SortDirection::Ascending {
            "▲"
        } else {
            "▼"
        }
    }

    fn to_string_with_arrow(self, sorted_column: Column, sort_direction: SortDirection) -> String {
        format!(
            "{}{}",
            self,
            self.sort_arrow_str(sorted_column, sort_direction)
        )
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Column::Pid => "pid",
            Column::Name => "name",
            Column::Cpu => "cpu",
            Column::Memory => "mem",
            Column::DiskRead => "disk r/s",
            Column::DiskWrite => "disk w/s",
        })
    }
}

pub struct Processes<'b> {
    processes: Vec<ProcessInfo>,
    style: Style,
    block: Option<Block<'b>>,

    sort_column: Column,
    sort_direction: SortDirection,
}

impl<'b> Processes<'b> {
    pub fn new(app: &App) -> Self {
        Self {
            processes: app.processes.clone(),
            style: Default::default(),
            block: Default::default(),

            sort_column: app.processes_sort_column,
            sort_direction: app.processes_sort_direction,
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn block(self, block: Block) -> Processes {
        Processes {
            block: Some(block),
            ..self
        }
    }
}

impl<'b> Widget for Processes<'b> {
    fn render(mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        self.processes.sort_by(|p1, p2| match self.sort_direction {
            SortDirection::Ascending => self.sort_column.compare_by(p1, p2),
            SortDirection::Descending => self.sort_column.compare_by(p1, p2).reverse(),
        });

        Table::new(self.processes.into_iter().map(|p| {
            Row::new(
                Column::ALL_COLUMNS
                    .iter()
                    .map(|c| c.extract_data_as_string(&p)),
            )
            .style(Style::default().fg(tui::style::Color::Blue))
        }))
        .column_spacing(1)
        .widths(&[Constraint::Ratio(1, 6); 6])
        .block(self.block.unwrap_or_default())
        .style(self.style)
        .header(
            Row::new(
                Column::ALL_COLUMNS
                    .iter()
                    .map(|c| c.to_string_with_arrow(self.sort_column, self.sort_direction)),
            )
            .style(
                Style::default()
                    .fg(tui::style::Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .render(area, buf);
    }
}
