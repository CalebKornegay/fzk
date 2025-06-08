mod app;
mod args;
mod interface;
mod ui;

use ratatui::crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::error::Error;
use scopeguard::defer;

use crate::interface::{ProcessMonitor, Monitor};

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let mut stdout = stdout();
    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = app.run(&mut terminal);

    defer!(
        app.join_threads();
    );

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{}", err.to_string());
    }

    // use clap::Parser;
    // let args = crate::args::Args::parse();

    // let mut monitor = Monitor::new(
    //     args.update_interval.unwrap_or(3.0),
    //     args.threshold.unwrap_or(0.4)
    // );

    // monitor.get_procs_from_system();

    // if let Some(res) = monitor.get_all_procs() {
    //     res.iter().for_each(|proclist| {
    //         proclist.iter().for_each(|proc| {
    //             println!("{} {} {} {}", proc.get_command(), proc.get_pid(), proc.get_mem(), proc.get_cpu());
    //         })
    //     })
    // }

    // if let Some(chrome) = monitor.current_procs.get("chrome.exe") {
    //     println!("found chrome, pids = {}", 
    //         chrome.iter().map(|p| p.get_pid().to_string()).collect::<Vec<String>>().join(" ")
    //     );
    // }

    // let res = monitor.current_procs.get("chrome.exe");
    // if let Some(chrome) = res {
    //     chrome.iter().for_each(|p| Monitor::kill_proc(p));
    // }

    // monitor.kill_proc_list("chrome.exe");

    // if let Some(chrome) = monitor.current_procs.get("chrome.exe") {
    //     println!("found chrome, pids = {}", 
    //         chrome.iter().map(|p| p.get_pid().to_string()).collect::<Vec<String>>().join(" ")
    //     );
    // }

    Ok(())
}
