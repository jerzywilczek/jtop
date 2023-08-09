use std::rc::Rc;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::app::App;

use self::{chart_wrapper::ChartWrapper, cpus_bars::CpusBars};

mod chart_wrapper;
mod cpus_bars;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let style = Style::default().fg(Color::Cyan);
    let block = Block::default()
        .borders(Borders::all())
        .border_type(BorderType::Rounded);

    let layout = Layout::default()
        .margin(0)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(frame.size());

    let cpus = split_cpus(layout[0], app.cpu_history.len());

    frame.render_widget(
        ChartWrapper::new(&app.cpu_history, |percentage, i| {
            format!("cpu{i}: {percentage:.1}%")
        })
        .style(style)
        .block(block.clone().title("cpu")),
        cpus[0],
    );

    frame.render_widget(
        CpusBars::new(app)
            .style(style)
            .block(block.clone().title("cpu")),
        cpus[1],
    );

    frame.render_widget(
        ChartWrapper::new(&[app.mem_history.clone()], |percentage, _| {
            format!("used mem: {percentage:.1}%")
        })
        .style(style)
        .block(block.title("mem")),
        layout[1],
    );
}

fn split_cpus(area: Rect, _cpus: usize) -> Rc<[Rect]> {
    Layout::default()
        .margin(0)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area)
}
