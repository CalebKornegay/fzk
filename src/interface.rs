use std::{cmp::Ordering, process::Command, u64};
use rust_fuzzy_search::fuzzy_search_threshold;

pub trait ProcessMonitor {
    fn get_procs_from_system(&mut self) -> ();
    fn kill_proc(&mut self, proc: &Process) -> ();
    fn kill_proc_list(&mut self, name: &str) -> ();
    fn get_procs_by_name_fuzzy(&self, search: &str, search_pid: bool) -> Option<Vec<Process>>;
    fn get_all_procs(&self) -> Option<Vec<Process>>;
}

// #[cfg(target_os = "windows")]
// const KILL_COMMAND: &'static str = "taskkill /T";
// #[cfg(any(target_os = "linux", target_os = "macos"))]
// const KILL_COMMAND: &'static str = "kill";

// #[cfg(target_os = "windows")]
// const UPDATE_COMMAND: &'static str = "tasklist /NH /FO TABLE";
// #[cfg(any(target_os = "linux", target_os = "macos"))]
// const UPDATE_COMMAND: &'static str = "ps -A --format comm,pid,%mem,%cpu";

#[cfg(target_os = "windows")]
pub const HEADERS: [&'static str; 3] = ["Command", "PID", "Memory Usage"];
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const HEADERS: [&'static str; 4] = ["Command", "PID", "Memory Usage (%)", "CPU Usage (%)"];

#[derive(Clone)]
pub struct Process {
    command: String,
    pid: u64,
    mem: String,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    cpu: String
}

impl Process {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            pid: u64::MAX,
            mem: String::new(),
             #[cfg(any(target_os = "linux", target_os = "macos"))]
            cpu: String::new()
        }
    }

    pub fn get_command(&self) -> &str {
        self.command.as_str()
    }

    pub fn get_pid(&self) -> u64 {
        self.pid
    }

    pub fn get_mem(&self) -> &str {
        &self.mem
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn get_cpu(&self) -> &str {
        &self.cpu
    }
}

pub struct Monitor {
    interval: f32,
    threshold: f32,
    num_matches: usize,
    current_procs: Vec<Process>
}

impl Monitor {
    pub fn new(inter: f32, thres: f32, num: usize) -> Self {
        Self {
            interval: inter.max(1.0),
            threshold: thres.max(0.0).min(1.0),
            num_matches: num.max(1),
            current_procs: Vec::new(),
        }
    }

    pub fn get_interval(&self) -> f32 {
        self.interval
    }

    #[cfg(debug_assertions)]
    pub fn print_all_procs(&self) -> () {
        self.current_procs.iter()
                .for_each(|proc| {
                    println!("{} {}", proc.command, proc.pid);
                });
    }
}

impl ProcessMonitor for Monitor {
    fn get_all_procs(&self) -> Option<Vec<Process>> {
        if self.current_procs.len() == 0 {
            None
        } else {
            Some(
                self.current_procs.clone()
            )
        }
    }

    fn get_procs_by_name_fuzzy(&self, search: &str, search_pid: bool) -> Option<Vec<Process>> {
        let procs =
            self.current_procs
            .iter()
            .map(|proc| {
                if search_pid {
                    proc.get_pid().to_string()
                } else {
                    proc.get_command().replace(".exe", "")
                }
            })
            .collect::<Vec<String>>();
        let refs = procs
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();

        let mut matches = fuzzy_search_threshold(search, &refs, self.threshold);
        matches
            .sort_by(|(_, score1), (_, score2)| {
                if score1 > score2 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
        let matches = 
            matches
            .iter()
            .map(|&(key, _)| key)
            .take(self.num_matches)
            .collect::<Vec<&str>>();

        if matches.len() == 0 {
            None
        } else {
            let mut ret: Vec<Process> = Vec::new();
            matches
                .iter()
                .for_each(|&p| {
                    if let Some(spot) = self.current_procs
                        .iter()
                        .position(|proc| {
                            if search_pid {
                                proc.get_pid().to_string() == p
                            } else {
                                proc.get_command().replace(".exe", "") == p
                            }
                        }) {
                            ret.push(self.current_procs[spot].clone());
                        }
                });

            Some(
                ret
            )
        }
    }

    #[cfg(target_os = "windows")]
    fn get_procs_from_system(&mut self) -> () {
        // Get the current list of processes
        let output = Command::new("tasklist")
            .args("/NH /FO TABLE".split(" "))
            .output()
            .expect("Failed to exec tasklist");

        // Check to see if the command executed successfully
        if !output.status.success() {
            return;
        }
        let Ok(res) = String::from_utf8(output.stdout) else {
            return;
        };

        // Clean out the old processes since we have a new list
        self.current_procs.clear();

        res.lines().for_each(|line| {
            // Iterate over every task and insert the process into the vector attached to that command (includes children)
            let mut p: Process = Process::new();
            let mut units: &str = "";

            // The columns are gotten from TABLE format in tasklist
            line.split_ascii_whitespace()
                .enumerate()
                .for_each(|(i, col)| {
                    match i {
                        0 => p.command = col.to_string(),
                        1 => p.pid = col.parse::<u64>().unwrap_or(u64::MAX),
                        4 => p.mem = col.to_string(),
                        5 => units = col,
                        _ => (),
                    }
                });

            if p.pid != u64::MAX {
                // Add the bytes units to the number
                p.mem.push_str(" ");
                p.mem.push_str(units);
                p.mem.push_str("iB");
                self.current_procs.push(p);
            }
        });
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn get_procs_from_system(&mut self) -> () {
        // Get the current list of processes
        let output = Command::new("ps")
            .args("-A --format comm,pid,%mem,%cpu".split(" "))
            .output()
            .expect("Failed to exec ps");

        // Check to see if the command executed successfully
        if !output.status.success() {
            return;
        }
        let Ok(res) = String::from_utf8(output.stdout) else {
            return;
        };

        self.current_procs.clear();

        res.lines().skip(1).for_each(|line| {
            let mut p: Process = Process::new();
            let mut comm: String = String::new();

            line.split_ascii_whitespace().enumerate().for_each(|(i, col)| {
                match i {
                    0 => p.command = col.to_string(),
                    1 => p.pid = col.parse::<u64>().unwrap_or(u64::MAX),
                    2 => p.mem = col.to_string(),
                    3 => p.cpu = col.to_string(),
                    _ => (),
                }
            });

            self.current_procs.push(p);
        });
    }

    fn kill_proc_list(&mut self, name: &str) -> () {
        let proc_list = self.current_procs
            .iter()
            .filter(|proc| proc.get_command() == name)
            .map(|proc| proc.clone())
            .collect::<Vec<Process>>();
        proc_list
            .iter()
            .for_each(|p| self.kill_proc(&p));
    }

    #[cfg(target_os = "windows")]
    fn kill_proc(&mut self, proc: &Process) -> () {
        let res = Command::new("taskkill")
            .arg("/T")
            .arg("/F")
            .arg("/PID")
            .arg(proc.pid.to_string())
            .output();

        let Ok(output) = res else {
            return;
        };

        if !output.status.success() {
            return;
        }

        if let Some(spot) = self.current_procs
            .iter()
            .position(|p| p.get_pid() == proc.get_pid()) {
            self.current_procs.remove(spot);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn kill_proc(&mut self, proc: &Process) -> () {
        let res = Command::new("kill")
            .arg("-9")
            .arg(proc.pid.to_string())
            .output();

        let Ok(output) = res else {
            return;
        };

        if !output.status.success() {
            eprintln!("Not successful, pid = {}", proc.pid.to_string());
            return;
        }

        if let Some(spot) = self.current_procs
            .iter()
            .position(|p| p.get_pid() == proc.get_pid()) {
            self.current_procs.remove(spot);
        }
    }
}
