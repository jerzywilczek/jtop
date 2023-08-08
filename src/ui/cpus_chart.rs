use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};

use crate::app::{App, HISTORY_LEN};

pub struct CpusChart<'a> {
    data: Vec<Vec<(f64, f64)>>,
    style: Style,
    block: Option<Block<'a>>,
}

impl<'a> CpusChart<'a> {
    pub fn new(app: &App) -> Self {
        let data = app
            .history
            .iter()
            .map(|cpu| {
                (0..HISTORY_LEN)
                    .map(|x| x as f64)
                    .zip(cpu.iter().copied())
                    .collect()
            })
            .collect();

        Self {
            data,
            style: Style::default(),
            block: None,
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn block<'b>(self, block: Block<'b>) -> CpusChart<'b> {
        CpusChart {
            block: Some(block),
            ..self
        }
    }
}

impl<'a> Widget for CpusChart<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let colors = [
            Color::Blue,
            Color::Cyan,
            Color::Green,
            Color::Magenta,
            Color::Red,
            Color::Yellow,
        ]
        .iter()
        .cycle();

        let datasets = self
            .data
            .iter()
            .zip(colors)
            .enumerate()
            .map(|(i, (data, &color))| {
                Dataset::default()
                    .data(data)
                    .graph_type(GraphType::Line)
                    .marker(Marker::Braille)
                    .name(format!("cpu{i}: {:.1}%", data.last().unwrap().1))
                    .style(Style::default().fg(color))
            })
            .collect();

        let mut chart = Chart::new(datasets)
            .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
            .y_axis(
                Axis::default()
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Span::raw(""),
                        Span::raw("20"),
                        Span::raw("40"),
                        Span::raw("60"),
                        Span::raw("80"),
                        Span::raw("100"),
                    ])
                    .labels_alignment(Alignment::Right),
            )
            .hidden_legend_constraints((Constraint::Percentage(80), Constraint::Percentage(80)))
            .style(self.style);

        if let Some(block) = self.block {
            chart = chart.block(block);
        }

        chart.render(area, buf);
    }
}
