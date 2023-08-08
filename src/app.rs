use std::{error, collections::VecDeque, time::Instant};

use sysinfo::{System, SystemExt, CpuExt};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub const HISTORY_LEN: usize = 64;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub history: Vec<VecDeque<f64>>,

    pub system: sysinfo::System,

    last_refresh: Instant,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut system = sysinfo::System::new();
        system.refresh_cpu();
        let last_refresh = Instant::now();

        let len = system.cpus().len();
        
        let history = vec![vec![0.0; HISTORY_LEN].into(); len].into();

        Self {
            running: true,
            history,
            last_refresh,
            system,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if self.last_refresh.elapsed() >= System::MINIMUM_CPU_UPDATE_INTERVAL {
            self.system.refresh_cpu();
            self.last_refresh = Instant::now();

            self.history.iter_mut().zip(self.system.cpus()).for_each(|(history, cpu)| {
                history.pop_front();
                history.push_back(cpu.cpu_usage() as f64)
            });
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
