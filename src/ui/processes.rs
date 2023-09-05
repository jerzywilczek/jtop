use std::cmp::Ordering;

use fuzzy_matcher::FuzzyMatcher;
use tui::{
    prelude::*,
    widgets::{block::Title, Block, Row, Table, Widget},
};

use crate::app::{App, InputState, MemPrefix, ProcessInfo};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Column {
    Pid,
    Name,
    #[default]
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
        match self {
            Column::Pid => info.pid.to_string(),
            Column::Name => info.name.clone(),
            Column::Cpu => format!("{:.01}%", info.cpu),
            Column::Memory => MemPrefix::best_string(info.mem as f64),
            Column::DiskRead => MemPrefix::best_string(info.disk_r as f64),
            Column::DiskWrite => MemPrefix::best_string(info.disk_w as f64),
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

    fn sort_arrow_str(&self, sorting: &InputState) -> &str {
        if let InputState::ProcessesSortSelection { column, direction } = sorting {
            if self != column {
                return "";
            }

            if *direction == SortDirection::Ascending {
                return "▲";
            } else {
                return "▼";
            }
        }

        ""
    }

    fn line_with_arrow(&self, sorting: &InputState) -> Line {
        let arrow = self.sort_arrow_str(sorting).into();
        let highlight_style = match sorting {
            InputState::ProcessesSortSelection { .. } => {
                Style::default().add_modifier(Modifier::UNDERLINED)
            }
            InputState::ProcessesSearch { .. } => Style::default(),
        };

        match self {
            Column::Pid => vec![Span::styled("p", highlight_style), "id".into(), arrow],
            Column::Name => vec![Span::styled("n", highlight_style), "ame".into(), arrow],
            Column::Cpu => vec![Span::styled("c", highlight_style), "pu".into(), arrow],
            Column::Memory => vec![Span::styled("m", highlight_style), "em".into(), arrow],
            Column::DiskRead => vec![
                "disk ".into(),
                Span::styled("r", highlight_style),
                "/s".into(),
                arrow,
            ],
            Column::DiskWrite => vec![
                "disk ".into(),
                Span::styled("w", highlight_style),
                "/s".into(),
                arrow,
            ],
        }
        .into()
    }
}

pub struct Processes<'b> {
    processes: Vec<ProcessInfo>,
    style: Style,
    block: Option<Block<'b>>,

    sorting: InputState,
}

impl<'b> Processes<'b> {
    pub fn new(app: &App) -> Self {
        Self {
            processes: app.processes.clone(),
            style: Default::default(),
            block: Default::default(),

            sorting: app.input_state.clone(),
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
        match &self.sorting {
            InputState::ProcessesSortSelection { column, direction } => {
                self.processes.sort_by(|p1, p2| match direction {
                    SortDirection::Ascending => column.compare_by(p1, p2),
                    SortDirection::Descending => column.compare_by(p1, p2).reverse(),
                });
            }
            InputState::ProcessesSearch { search, .. } => {
                let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

                self.processes.sort_by_key(|p| {
                    if let Some((score, _)) = matcher.fuzzy_indices(&p.name, search) {
                        -score
                    } else {
                        i64::MAX
                    }
                })
            }
        }

        let bottom_title = match &self.sorting {
            InputState::ProcessesSortSelection { .. } => " press / to search ".to_string(),
            InputState::ProcessesSearch { search, .. } => format!(" searched: {search}_ "),
        };

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
        .block(
            self.block
                .unwrap_or_default()
                .title(Title::from(bottom_title).position(tui::widgets::block::Position::Bottom)),
        )
        .style(self.style)
        .header(
            Row::new(
                Column::ALL_COLUMNS
                    .iter()
                    .map(|c| c.line_with_arrow(&self.sorting)),
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
