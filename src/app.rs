use std::{collections::VecDeque, error, time::Instant};

use sysinfo::{CpuExt, System, SystemExt};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub const HISTORY_LEN: usize = 64;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub cpu_history: Vec<VecDeque<f64>>,
    pub mem_history: VecDeque<f64>,

    pub system: sysinfo::System,

    last_refresh: Instant,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut system = sysinfo::System::new();
        system.refresh_cpu();
        system.refresh_memory();
        let last_refresh = Instant::now();

        let len = system.cpus().len();

        let cpu_history = vec![vec![0.0; HISTORY_LEN].into(); len].into();
        let mem_history = vec![0.0; HISTORY_LEN].into();

        Self {
            running: true,
            cpu_history,
            mem_history,
            last_refresh,
            system,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if self.last_refresh.elapsed() >= System::MINIMUM_CPU_UPDATE_INTERVAL {
            self.system.refresh_cpu();
            self.last_refresh = Instant::now();

            self.cpu_history
                .iter_mut()
                .zip(self.system.cpus())
                .for_each(|(history, cpu)| {
                    history.pop_front();
                    history.push_back(cpu.cpu_usage() as f64)
                });
        }

        self.system.refresh_memory();
        self.mem_history.pop_front();
        self.mem_history.push_back(
            self.system.used_memory() as f64 / self.system.total_memory() as f64 * 100.0,
        );
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
