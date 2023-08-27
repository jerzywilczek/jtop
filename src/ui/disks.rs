use tui::{
    prelude::*,
    widgets::{Block, Widget},
};

use crate::app::App;

#[cfg(target_os = "windows")]
use tui::widgets::Paragraph;

#[cfg(not(target_os = "windows"))]
use super::chart_wrapper::ChartWrapper;

#[cfg(not(target_os = "windows"))]
fn to_mb(sectors: usize) -> f64 {
    // FIXME: some disks have sector size != 512
    sectors as f64 * 512.0 / 1_000_000.0
}

pub struct Disks<'a> {
    #[cfg(not(target_os = "windows"))]
    chart: ChartWrapper<'a>,

    #[cfg(target_os = "windows")]
    paragraph: Paragraph<'a>,
}

impl<'a> Disks<'a> {
    #[cfg(not(target_os = "windows"))]
    pub fn new(app: &App) -> Self {
        let data = app
            .disks
            .values()
            .flat_map(|(_, q)| {
                let mut r = Vec::with_capacity(q.len());
                let mut w = Vec::with_capacity(q.len());

                for info in q.iter() {
                    r.push(to_mb(info.r_sectors));
                    w.push(to_mb(info.w_sectors));
                }

                [r.into(), w.into()]
            })
            .collect::<Vec<_>>();

        let &max = data
            .iter()
            .flatten()
            .max_by(|&&f1: &&f64, &f2| f1.total_cmp(f2))
            .unwrap_or(&1.0);
        let names = app.disks.keys().cloned().collect::<Vec<_>>();

        let chart = ChartWrapper::new(
            &data,
            Box::new(move |v, i| {
                format!(
                    "{} {}: {v:.02}MB/s",
                    names[i / 2],
                    if i % 2 == 0 { "r" } else { "w" }
                )
            }),
            [0.0, max],
        );

        Self { chart }
    }

    #[cfg(target_os = "windows")]
    pub fn new(_app: &App) -> Self {
        Self {
            paragraph: Paragraph::new("disks view not supported on windows ;(")
                .alignment(Alignment::Center),
        }
    }

    pub fn style(self, style: Style) -> Self {
        Self {
            #[cfg(not(target_os = "windows"))]
            chart: self.chart.style(style),

            #[cfg(target_os = "windows")]
            paragraph: self.paragraph.style(style),
        }
    }

    pub fn block(self, block: Block<'a>) -> Disks {
        Disks {
            #[cfg(not(target_os = "windows"))]
            chart: self.chart.block(block),

            #[cfg(target_os = "windows")]
            paragraph: self.paragraph.block(block),
        }
    }
}

impl<'a> Widget for Disks<'a> {
    #[cfg(not(target_os = "windows"))]
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.chart.render(area, buf);
    }

    #[cfg(target_os = "windows")]
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.render(area, buf);
    }
}
