use std::{collections::BTreeMap, collections::VecDeque, error, time::Instant};

use regex::Regex;
use sysinfo::{CpuExt, Pid, Process, ProcessExt, System, SystemExt};
use systemstat::{BlockDeviceStats, Platform};

use crate::{config::Config, ui::processes::Column};

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
    fn new(proc: &Process, cpu_amount: usize) -> Self {
        Self {
            pid: proc.pid(),
            cpu: proc.cpu_usage() as f64 / cpu_amount as f64,
            mem: proc.memory(),
            name: proc.name().to_string(),
            // FIXME: as per documentation, this is incorrect for FreeBSD and Windows
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
                Regex::new(r"^nvme[0-9]+n[0-9]+$").unwrap(),
                Regex::new(r"^sd[a-z]+$").unwrap(),
                Regex::new(r"^hd[a-z]+$").unwrap(),
                Regex::new(r"^disk[0-9]+$").unwrap(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputState {
    ProcessesSortSelection {
        column: crate::ui::processes::Column,
        direction: crate::ui::processes::SortDirection,
    },
    ProcessesSearch {
        old_column: Option<crate::ui::processes::Column>,
        old_direction: Option<crate::ui::processes::SortDirection>,
        search: String,
    },
}

impl Default for InputState {
    fn default() -> Self {
        let column = Column::default();
        let direction = column.default_sort_direction();
        Self::ProcessesSortSelection { column, direction }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemPrefix {
    Byte = 0,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
}

impl MemPrefix {
    pub const MULTIPLIER: u64 = 1024;
    pub const PREFIXES: &[Self] = &[
        MemPrefix::Byte,
        MemPrefix::Kilo,
        MemPrefix::Mega,
        MemPrefix::Giga,
        MemPrefix::Tera,
        MemPrefix::Peta,
    ];

    pub fn next(self) -> Self {
        match self {
            MemPrefix::Byte => MemPrefix::Kilo,
            MemPrefix::Kilo => MemPrefix::Mega,
            MemPrefix::Mega => MemPrefix::Giga,
            MemPrefix::Giga => MemPrefix::Tera,
            MemPrefix::Tera => MemPrefix::Peta,
            MemPrefix::Peta => MemPrefix::Peta,
        }
    }

    pub fn prefix(self) -> char {
        match self {
            MemPrefix::Byte => 'B',
            MemPrefix::Kilo => 'K',
            MemPrefix::Mega => 'M',
            MemPrefix::Giga => 'G',
            MemPrefix::Tera => 'T',
            MemPrefix::Peta => 'P',
        }
    }

    pub fn find_best(mut bytes: f64) -> (f64, Self) {
        for &prefix in Self::PREFIXES {
            if bytes < Self::MULTIPLIER as f64 {
                return (bytes, prefix);
            }

            bytes /= Self::MULTIPLIER as f64;
        }

        (
            bytes * Self::MULTIPLIER as f64,
            *Self::PREFIXES.last().unwrap(),
        )
    }

    pub fn best_string(bytes: f64) -> String {
        let (mem, prefix) = Self::find_best(bytes);

        format!("{mem:.1}{}", prefix.prefix())
    }

    pub fn convert(self, bytes: f64) -> f64 {
        bytes / (Self::MULTIPLIER as f64).powi(self as i32)
    }
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub input_state: InputState,
    pub config: Config,

    pub cpu_history: Vec<VecDeque<f64>>,
    pub mem_history: VecDeque<f64>,
    pub mem_total: f64,
    pub mem_prefix: MemPrefix,

    pub processes: Vec<ProcessInfo>,
    pub disks: BTreeMap<String, (DiskInfo, VecDeque<DiskInfo>)>,

    pub system: sysinfo::System,
    pub systemstat: systemstat::System,

    last_refresh: Instant,
    disk_regexes: DiskRegexes,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(config: Config) -> Self {
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

        let cpu_history = vec![vec![0.0; HISTORY_LEN].into(); len];

        let mem_history = vec![0.0; HISTORY_LEN].into();
        let (mem_total, mem_prefix) = MemPrefix::find_best(system.total_memory() as f64);

        let processes = system
            .processes()
            .values()
            .map(|p| ProcessInfo::new(p, len))
            .collect();

        Self {
            running: true,
            input_state: Default::default(),
            config,
            cpu_history,
            mem_history,
            mem_total,
            mem_prefix,
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
        self.mem_history
            .push_back(self.mem_prefix.convert(self.system.used_memory() as f64));

        self.system.refresh_processes();
        self.processes = self
            .system
            .processes()
            .values()
            .map(|p| ProcessInfo::new(p, self.system.cpus().len()))
            .collect();

        self.systemstat
            .block_device_statistics()
            .unwrap_or_default()
            .into_iter()
            .filter(|(n, _)| self.disk_regexes.is_disk(n))
            .map(|(n, d)| (n, DiskInfo::new(&d)))
            .for_each(|(name, current)| {
                let (prev, history) = self.disks.entry(name).or_insert((
                    Default::default(),
                    vec![Default::default(); HISTORY_LEN].into(),
                ));
                history.pop_front();
                history.push_back(DiskInfo {
                    r_sectors: current.r_sectors - prev.r_sectors,
                    w_sectors: current.w_sectors - prev.w_sectors,
                });
                *prev = current;
            });
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
