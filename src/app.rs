use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::{Duration, SystemTime}};

use clap::Parser;
use ratatui::{crossterm::event::{KeyEventKind, KeyModifiers, MouseEventKind}, layout::{Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::interface::{Monitor, Process, ProcessMonitor};
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
                        args.threshold.unwrap_or(0.3)
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
        let mut num_lines: usize = 0;

        let keybinds_text = vec![
            "[ctrl+h] help",
            "[ctrl+(q|c)] quit",
            "[ctrl+k] kill process",
            "[ctrl+b] clear search",
            "[ctrl+r] reset scroll"
        ];

        // Background thread to collect data
        
        loop {
            terminal.draw(|frame| {
                let current_area = frame.area();
                num_lines = current_area.height as usize - 3;

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

                let proc_names = Paragraph::new(
                    current_procs
                    .iter()
                    .flat_map(|proc_l| {
                        proc_l.iter().map(|proc| {
                            proc.get_command()
                        })
                        .collect::<Vec<&str>>()
                    })
                    .skip(self.current_line)
                    .take(num_lines)
                    .enumerate()
                    .map(|(i, command)| {
                        if i == self.pointer {
                            Line::styled(command, Style::new().fg(Color::LightBlue))
                        } else {
                            Line::from(command)
                        }
                    })
                    .collect::<Vec<Line>>()
                );
                let proc_names_rect = Rect::new(1, 1, current_area.width / 4, num_lines as u16);

                frame.render_widget(proc_names, proc_names_rect);

                let current_search = Paragraph::new(search_input.clone())
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
                        .title("Current Search")
                        .title_alignment(ratatui::layout::Alignment::Left)
                        .title_style(Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)))
                        .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)))
                    );

                let search_rect = Rect::new(0, num_lines as u16, current_area.width / 4, 3);

                let help_text = Paragraph::new(keybinds_text.join("   "))
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
                        .title("Keybinds")
                        .title_alignment(ratatui::layout::Alignment::Left)
                        .title_style(Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)))
                        .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)))
                    )
                    .alignment(ratatui::layout::Alignment::Center);
                let help_rect = Rect::new(current_area.width / 4, num_lines as u16, current_area.width - current_area.width / 4, 3);

                let proc_list_block = Ui::generate_block(String::from("Current Processes"));
                let proc_rect =  Rect::new(0, 0, current_area.width, num_lines as u16);

                frame.render_widget(help_text, help_rect);
                frame.render_widget(proc_list_block, proc_rect);
                frame.render_widget(current_search, search_rect);
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
                                    KeyCode::Char('h') => show_help = !show_help,
                                    KeyCode::Char('k') => {
                                        self.monitor.lock()
                                            .unwrap()
                                            .kill_proc(
                                                &current_procs.iter()
                                                .flatten()
                                                .skip(self.current_line + self.pointer)
                                                .take(1)
                                                .collect::<Vec<&Process>>()
                                                [0]
                                            );
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

                                        self.current_line = std::cmp::min(
                                            self.current_line + 1, 
                                            count.saturating_sub(num_lines
                                                .saturating_sub(2))
                                        );

                                        if count - self.current_line < num_lines {
                                            self.pointer = std::cmp::min(
                                                self.pointer + 1,
                                                std::cmp::min(count, num_lines)
                                                .saturating_sub(1)
                                            );
                                        }
                                    },
                                    KeyCode::Up => {
                                        self.current_line = 
                                            self.current_line.saturating_sub(1);
                                        self.pointer = 
                                            self.pointer.saturating_sub(1);
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

                                    self.current_line = std::cmp::min(
                                        self.current_line + 1, 
                                        count.saturating_sub(num_lines
                                            .saturating_sub(2))
                                    );

                                    if count - self.current_line < num_lines {
                                        self.pointer = std::cmp::min(
                                            self.pointer + 1,
                                            std::cmp::min(count, num_lines)
                                            .saturating_sub(1)
                                        );
                                    }
                                },
                                MouseEventKind::ScrollUp => {
                                    self.current_line = 
                                        self.current_line.saturating_sub(1);
                                    self.pointer = 
                                            self.pointer.saturating_sub(1);
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
