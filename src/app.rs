use std::{collections::BTreeMap, collections::VecDeque, error, time::Instant};

use regex::Regex;
use sysinfo::{CpuExt, Pid, Process, ProcessExt, System, SystemExt};
use systemstat::{BlockDeviceStats, Platform};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct DiskInfo {
    pub r_sectors: usize,
    pub w_sectors: usize,
}

impl DiskInfo {
    fn new(stats: &BlockDeviceStats) -> Self {
        Self {
            r_sectors: stats.read_sectors,
            w_sectors: stats.write_sectors,
        }
    }
}

struct DiskRegexes {
    disks: Vec<Regex>,
}

impl DiskRegexes {
    fn is_disk(&self, name: &str) -> bool {
        self.disks.iter().any(|r| r.is_match(name))
    }
}

impl Default for DiskRegexes {
    fn default() -> Self {
        Self {
            disks: vec![
                Regex::new(r"nvme[0-9]*n[0-9]*$").unwrap(),
                Regex::new(r"sd[a-z]*$").unwrap(),
                Regex::new(r"hd[a-z]*$").unwrap(),
                Regex::new(r"[A-Z]:\\").unwrap(),
            ]
        }
    }
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub cpu_history: Vec<VecDeque<f64>>,
    pub mem_history: VecDeque<f64>,
    pub processes: Vec<ProcessInfo>,
    pub disks: BTreeMap<String, (DiskInfo, VecDeque<DiskInfo>)>,

    pub system: sysinfo::System,
    pub systemstat: systemstat::System,

    last_refresh: Instant,
    disk_regexes: DiskRegexes,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut system = sysinfo::System::new();
        let systemstat = systemstat::System::new();
        system.refresh_cpu();
        system.refresh_memory();
        system.refresh_processes();
        let last_refresh = Instant::now();
        let disk_regexes = DiskRegexes::default();

        let len = system.cpus().len();

        let disks = systemstat
            .block_device_statistics()
            .unwrap_or_default()
            .into_iter()
            .filter(|(n, _)| disk_regexes.is_disk(n))
            .map(|(n, d)| {
                let q: VecDeque<_> = vec![DiskInfo::default(); HISTORY_LEN].into();
                
                (n, (DiskInfo::new(&d), q))
            })
            .collect::<BTreeMap<_, _>>();

        let cpu_history = vec![vec![0.0; HISTORY_LEN].into(); len].into();
        let mem_history = vec![0.0; HISTORY_LEN].into();
        let processes = system.processes().values().map(ProcessInfo::new).collect();

        Self {
            running: true,
            cpu_history,
            mem_history,
            last_refresh,
            system,
            systemstat,
            processes,
            disks,
            disk_regexes: Default::default(),
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

        self.systemstat
            .block_device_statistics()
            .unwrap_or_default()
            .into_iter()
            .filter(|(n, _)| self.disk_regexes.is_disk(n))
            .map(|(n, d)| (n, DiskInfo::new(&d)))
            .for_each(|(name, current)| {
                let (prev, history) = self.disks.entry(name).or_insert((Default::default(), vec![Default::default(); HISTORY_LEN].into()));
                history.pop_front();
                history.push_back(DiskInfo { r_sectors: current.r_sectors - prev.r_sectors, w_sectors: current.w_sectors - prev.w_sectors });
                *prev = current;
            });
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
