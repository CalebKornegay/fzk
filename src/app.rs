use clap::Parser;
use ratatui::{crossterm::event::{KeyEventKind, MouseEventKind}, layout::{Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::interface::Monitor;
use crate::args::Args;
use crate::ui::Ui;


pub struct App {
    current_line: usize,
    monitor: Monitor,
}

impl App {
    pub fn new() -> Self {
        let args = Args::parse();

        Self {
            current_line: 0,
            monitor: Monitor::new(
                args.update_interval.unwrap_or(3.0),
                args.threshold.unwrap_or(0.6),
                args.num_matches.unwrap_or(20)
            )
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        let mut show_help: bool = false;
        let keybinds_text = vec![
            "[h] help",
            "[q] quit",
        ];
        
        loop {
            terminal.draw(|frame| {
                // Show the help screen if 'h' was pressed
                if show_help {
                    Ui::show_help(frame, &keybinds_text);
                    return;
                }
            })?;

            // While loop so that we don't re-render the screen when nothing would've changed
            let mut should_break = false;
            while !should_break {
                let event: Event = event::read()?;
                match event {
                    Event::Key(key) =>  {
                        // Don't render the key event twice
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }

                        // Enable quit, refresh, and vertical and horizontal scroll
                        match key.code {
                            KeyCode::Char('h') => {
                                show_help = !show_help;
                                break;
                            },
                            KeyCode::Char('q') => should_break = true,
                            KeyCode::Char('r') => {
                                self.current_line = 0;
                                break;
                            },
                            KeyCode::Up => {
                                // Don't scroll past beginning
                                if self.current_line > 0 {
                                    self.current_line -= 1;
                                    break;
                                }
                            },
                            KeyCode::Down => {
                                self.current_line += 1;
                                break;
                            }
                            _ => continue
                        }
                    },
                    Event::Mouse(e) => {
                        match e.kind {
                            MouseEventKind::ScrollDown => {
                                self.current_line += 1;
                                break;
                            },
                            MouseEventKind::ScrollUp => {
                                // Don't scroll past beginning
                                if self.current_line > 0 {
                                    self.current_line -= 1;
                                    break;
                                }
                            }
                            _ => continue
                        }
                    }
                    _ => continue
                }
            }

            if should_break {
                break;
            }
        }
        
        Ok(())
    }
}
