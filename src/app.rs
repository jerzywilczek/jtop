use std::{collections::VecDeque, error, time::Instant};

use sysinfo::{CpuExt, Pid, Process, ProcessExt, System, SystemExt};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub const HISTORY_LEN: usize = 64;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub cpu: f64,
    pub mem: u64,
    pub name: String,
    pub disk_r: u64,
    pub disk_w: u64,
}

impl ProcessInfo {
    fn new(proc: &Process) -> Self {
        Self {
            pid: proc.pid(),
            cpu: proc.cpu_usage() as f64,
            mem: proc.memory(),
            name: proc.name().to_string(),
            disk_r: proc.disk_usage().read_bytes,
            disk_w: proc.disk_usage().written_bytes,
        }
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub cpu_history: Vec<VecDeque<f64>>,
    pub mem_history: VecDeque<f64>,
    pub processes: Vec<ProcessInfo>,

    pub system: sysinfo::System,

    last_refresh: Instant,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut system = sysinfo::System::new();
        system.refresh_cpu();
        system.refresh_memory();
        system.refresh_processes();
        let last_refresh = Instant::now();

        let len = system.cpus().len();

        let cpu_history = vec![vec![0.0; HISTORY_LEN].into(); len].into();
        let mem_history = vec![0.0; HISTORY_LEN].into();
        let processes = system.processes().values().map(ProcessInfo::new).collect();

        Self {
            running: true,
            cpu_history,
            mem_history,
            last_refresh,
            system,
            processes,
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

        self.system.refresh_processes();
        self.processes = self
            .system
            .processes()
            .values()
            .map(ProcessInfo::new)
            .collect();
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
