use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::{Duration, SystemTime}};

use clap::Parser;
use ratatui::{crossterm::event::{KeyEventKind, KeyModifiers, MouseEventKind}, layout::{Constraint, Layout, Margin, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::interface::{Monitor, Process, ProcessMonitor, HEADERS};
use crate::args::Args;
use crate::ui::Ui;


pub struct App {
    should_die: Arc<Mutex<bool>>,
    current_line: usize,
    pointer: usize,
    monitor: Arc<Mutex<Monitor>>,
    threads: Vec<JoinHandle<()>>
}

impl App {
    pub fn new() -> Self {
        let args = Args::parse();
        let mut ret = Self {
            should_die: Arc::new(Mutex::new(false)),
            current_line: 0,
            pointer: 0,
            monitor: Arc::new(
                Mutex::new(
                    Monitor::new(
                        args.update_interval.unwrap_or(3.0),
                        args.threshold.unwrap_or(0.3),
                        args.num_matches.unwrap_or(20)
                    )
                )
            ),
            threads: Vec::new()
        };
        
        ret.collect_data();
        ret
    }

    pub fn join_threads(self) {
        for thread  in self.threads {
            let _ = thread.join().unwrap();
        }
    }

    fn collect_data(&mut self) {
        let mon = Arc::clone(&self.monitor);
        let die = Arc::clone(&self.should_die);

        let data_thread = thread::spawn(move || {
            let interval = {
                mon.lock().unwrap().get_interval()
            };

            loop {
                {
                    if *die.lock().unwrap() {
                        break;
                    }
                }

                let start_time = SystemTime::now();
                {
                    mon.lock().unwrap().get_procs_from_system();
                }

                if let Ok(elapsed_time) = start_time.elapsed() {
                    let time_to_sleep = 
                        Duration::from_secs_f32(
                            interval
                        )
                        .saturating_sub(elapsed_time);
                    
                    if !time_to_sleep.is_zero() {
                        thread::sleep(time_to_sleep);
                    }
                }
            }
        });
        self.threads.push(data_thread);
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        let mut show_help = false;
        let mut search_input = String::new();
        let mut current_procs: Vec<Vec<Process>> = Vec::new();
        let mut proc_list_size: usize = 0;
        let mut num_lines: usize = 0;
        let mut current_process: Process = Process::new();
        const HEADER_LEN: usize = HEADERS.len();

        let keybinds_text = vec![
            "[ctrl+h] help",
            "[ctrl+(q|c)] quit",
            "[ctrl+k] kill process",
            "[ctrl+b] clear search",
        ];

        // Background thread to collect data
        
        loop {
            terminal.draw(|frame| {
                let current_area = frame.area();
                proc_list_size = current_area.height as usize - 3;

                if current_area.height < 6 {
                   return ();
                }

                // Show the help screen if 'ctrl+h' was pressed
                if show_help {
                    Ui::show_help(frame, &keybinds_text);
                    return;
                }
                
                match self.monitor.try_lock() {
                    Ok(guard) => {
                        if search_input.len() > 0 {
                            current_procs = guard.get_procs_by_name_fuzzy(&search_input)
                                .unwrap_or(Vec::new());
                        } else {
                            current_procs = guard.get_all_procs()
                                .unwrap_or(Vec::new());
                        }
                    },
                    _ => ()
                }

                let mut proc_info: Vec<Vec<Line>> = vec![Vec::new(); HEADER_LEN];

                current_procs
                    .iter()
                    .flat_map(|proc_l| {
                        proc_l.iter().map(|proc| {
                            proc
                        })
                        .collect::<Vec<&Process>>()
                    })
                    .skip(self.current_line)
                    .take(num_lines)
                    .enumerate()
                    .for_each(|(i, proc)|{
                        if i == self.pointer {
                            let style = Style::new().fg(Color::LightBlue);
                            current_process = proc.clone();

                            proc_info[0].push(
                                Line::styled(
                                    proc.get_command(), 
                                    style.clone()
                                )
                            );
                            proc_info[1].push(
                                Line::styled(
                                    proc.get_pid().to_string(),
                                    style.clone()
                                )
                            );
                            proc_info[2].push(
                                Line::styled(
                                    proc.get_mem(),
                                    style.clone()
                                )
                            );
                            #[cfg(any(target_os = "linux", target_os = "macos"))]
                            proc_info[3].push(
                                Line::styled(
                                    proc.get_cpu(),
                                    style
                                )
                            );
                        } else {
                            proc_info[0].push(Line::from(proc.get_command()));
                            proc_info[1].push(
                                Line::from(
                                    proc.get_pid().to_string(),
                                )
                            );
                            proc_info[2].push(
                                Line::from(
                                    proc.get_mem(),
                                )
                            );
                            #[cfg(any(target_os = "linux", target_os = "macos"))]
                            proc_info[3].push(
                                Line::from(
                                    proc.get_cpu(),
                                )
                            );
                        }
                    });

                let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
                        .title_alignment(ratatui::layout::Alignment::Left)
                        .title_style(Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)))
                        .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)));
                
                let current_search = Paragraph::new(search_input.clone())
                    .block(block.clone().title("Current Search"));
                let search_rect = Rect::new(0, proc_list_size as u16, current_area.width / 4, 3);

                let help_text = Paragraph::new(keybinds_text.join("   "))
                    .block(block.clone().title("Keybinds"))
                    .alignment(ratatui::layout::Alignment::Center);
                let help_rect = Rect::new(current_area.width / 4, proc_list_size as u16, current_area.width - current_area.width / 4, 3);

                let proc_list_block = Ui::generate_block(String::from("Current Processes"));
                let proc_rect =  Rect::new(0, 0, current_area.width, proc_list_size as u16);

                num_lines = proc_rect.inner(Margin::new(1,1)).height as usize;

                let proc_rects = Layout::horizontal(
                        proc_info.iter().map(|_| {
                            Constraint::Percentage(100 / proc_info.len() as u16)
                        })
                        .collect::<Vec<Constraint>>()
                    )
                    .areas::<HEADER_LEN>(proc_rect.inner(Margin::new(1, 1)));

                frame.render_widget(help_text, help_rect);
                frame.render_widget(proc_list_block, proc_rect);
                frame.render_widget(current_search, search_rect);

                proc_rects.iter().zip(proc_info).enumerate().for_each(|(i, (rect, info))| {
                    frame.render_widget(
                        Paragraph::new(info)
                        .block(
                            block.clone().title(HEADERS[i])
                        ), 
                        *rect
                    );
                });
            })?;

            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                if let Ok(event) = event::read() {
                    match event {
                        Event::Key(key) =>  {
                            // Don't render the key event twice
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }
                            
                            // Enable quit and show help and clearing the input buffer
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                match key.code {
                                    KeyCode::Char('b') => {
                                        if search_input.len() > 0 {
                                            self.pointer = 0;
                                            self.current_line = 0;
                                        }
                                        search_input.clear();
                                    },
                                    KeyCode::Char('h') => {
                                        show_help = !show_help
                                    },
                                    KeyCode::Char('k') => {
                                        self.monitor.lock()
                                            .unwrap()
                                            .kill_proc(&current_process);
                                    },
                                    KeyCode::Char('r') => {
                                        self.current_line = 0;
                                        self.pointer = 0;
                                    },
                                    KeyCode::Char('q') | KeyCode::Char('c') => {
                                        *self.should_die.lock().unwrap() = true;
                                        return Ok(());
                                    }
                                    _ => ()
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char(char) => {
                                        search_input.push(char);
                                        self.pointer = 0;
                                        self.current_line = 0;
                                    },
                                    KeyCode::Backspace => {
                                        let _ = search_input.pop();
                                        self.pointer = 0;
                                        self.current_line = 0;
                                    },
                                    KeyCode::Down => {
                                        let count = current_procs
                                            .iter()
                                            .flatten()
                                            .count();

                                        let last_line = self.current_line;

                                        self.current_line = std::cmp::min(
                                            self.current_line + 1, 
                                            count.saturating_sub(
                                                num_lines
                                                .saturating_sub(2)
                                            )
                                        );

                                        // We didn't move down
                                        if self.current_line == last_line {
                                            self.pointer = std::cmp::min(
                                                self.pointer + 1,
                                                count.saturating_sub(
                                                    self.current_line
                                                    .saturating_add(1)
                                                )
                                            );
                                        }
                                    },
                                    KeyCode::Up => {
                                        let last_line = self.current_line;
                                        self.current_line = 
                                            self.current_line.saturating_sub(1);

                                        // We didn't move up
                                        if last_line == self.current_line {
                                            self.pointer = 
                                                self.pointer.saturating_sub(1);
                                        }
                                    },
                                    _ => ()
                                }
                            }
                        },
                        Event::Mouse(me) => {
                            match me.kind {
                                MouseEventKind::ScrollDown => {
                                    let count = current_procs
                                            .iter()
                                            .flatten()
                                            .count();

                                        let last_line = self.current_line;

                                        self.current_line = std::cmp::min(
                                            self.current_line + 1, 
                                            count.saturating_sub(
                                                num_lines
                                                .saturating_sub(2)
                                            )
                                        );

                                        // We didn't move down
                                        if self.current_line == last_line {
                                            self.pointer = std::cmp::min(
                                                self.pointer + 1,
                                                count.saturating_sub(
                                                    self.current_line
                                                    .saturating_add(1)
                                                )
                                            );
                                        }
                                },
                                MouseEventKind::ScrollUp => {
                                    let last_line = self.current_line;
                                        self.current_line = 
                                            self.current_line.saturating_sub(1);

                                        // We didn't move up
                                        if last_line == self.current_line {
                                            self.pointer = 
                                                self.pointer.saturating_sub(1);
                                        }
                                }
                                _ => ()
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}
