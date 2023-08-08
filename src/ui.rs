use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame, layout::{Layout, Direction, Constraint},
};

use crate::app::App;

use self::{cpus_chart::CpusChart, cpus_bars::CpusBars};

mod cpus_bars;
mod cpus_chart;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let style = Style::default().fg(Color::Cyan);
    let block = Block::default()
        .borders(Borders::all())
        .border_type(BorderType::Rounded);

    let layout = Layout::default()
        .margin(0)
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(3, 4), Constraint::Max(2 + app.history.len() as u16 / 4)])
        .split(frame.size());

    frame.render_widget(
        CpusChart::new(app)
            .style(style)
            .block(block.clone().title("cpu")),
        layout[0],
    );

    frame.render_widget(CpusBars::new(app).style(style).block(block.title("cpu")), layout[1])
}
