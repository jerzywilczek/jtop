use std::collections::VecDeque;

use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};

use crate::app::HISTORY_LEN;

pub struct ChartWrapper<'a, 'b> {
    data: Vec<Vec<(f64, f64)>>,
    style: Style,
    block: Option<Block<'b>>,
    label_generator: Box<dyn Fn(f64, usize) -> String + 'a>,
    range: [f64; 2],
    label_suffix: Option<char>,
}

impl<'a, 'b> ChartWrapper<'a, 'b> {
    pub fn new(
        data: &[VecDeque<f64>],
        label_generator: Box<dyn Fn(f64, usize) -> String + 'a>,
        range: [f64; 2],
    ) -> Self {
        let data = data
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
            label_generator,
            range,
            label_suffix: None,
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn block<'c>(self, block: Block<'c>) -> ChartWrapper<'a, 'c> {
        ChartWrapper {
            block: Some(block),
            ..self
        }
    }

    pub fn label_suffix(self, label_suffix: char) -> Self {
        Self {
            label_suffix: Some(label_suffix),
            ..self
        }
    }
}

impl<'a, 'b> Widget for ChartWrapper<'a, 'b> {
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
                    .name((self.label_generator)(data.last().unwrap().1, i))
                    .style(Style::default().fg(color))
            })
            .collect();

        let label_suffix = self
            .label_suffix
            .map(String::from)
            .unwrap_or_else(String::new);

        let mut chart = Chart::new(datasets)
            .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
            .y_axis(
                Axis::default()
                    .bounds(self.range)
                    .labels(vec![
                        Span::raw(""),
                        Span::raw(format!("{:.0}{label_suffix}", self.range[1] / 5.0)),
                        Span::raw(format!("{:.0}{label_suffix}", self.range[1] * 2.0 / 5.0)),
                        Span::raw(format!("{:.0}{label_suffix}", self.range[1] * 3.0 / 5.0)),
                        Span::raw(format!("{:.0}{label_suffix}", self.range[1] * 4.0 / 5.0)),
                        Span::raw(format!("{:.0}{label_suffix}", self.range[1])),
                    ])
                    .labels_alignment(Alignment::Right),
            )
            .hidden_legend_constraints((Constraint::Percentage(75), Constraint::Percentage(75)))
            .style(self.style);

        if let Some(block) = self.block {
            chart = chart.block(block);
        }

        chart.render(area, buf);
    }
}
