use ratatui::{layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Style}, text::Line, widgets::{Block, Borders, Paragraph}, Frame};

pub struct Ui;

impl Ui {
    pub fn show_help<'a>(frame: &mut Frame<'a>, keybinds_text: &Vec<&str>, tc: Color, bgc: Color) {
        frame.render_widget(
            Paragraph::new("")
                .block(
                    Self::generate_block(
                        String::from("Help"),
                        Some(tc),
                        Some(bgc)
                    )
            ),
            frame.area()
        );

        let style = Style::default().fg(tc);
                    
        let mut help_text = keybinds_text.clone().iter()
            .map(|&l| {
                if l == "[ctrl+h] help" {
                    Line::styled("[ctrl+h] to exit this screen", style.clone())
                } else {
                    Line::styled(l, style.clone())
                }
            })
            .collect::<Vec<Line>>();
        help_text.extend_from_slice(
            &[
                Line::styled("[ctrl+r] reset scroll", style.clone()),
                Line::styled("[\u{2195}] use arrow keys or mouse to move up and down", style.clone()),
                Line::styled("Enter characters to fuzzy search for processes", style),
            ]
        );

        let b = Self::center_rect(frame.area(), 
            Constraint::Length(help_text.iter()
                .map(|l| l.width()).max().unwrap() as u16),
            Constraint::Length(help_text.len() as u16 + 2));

        frame.render_widget(
            Paragraph::new(help_text), b
        );
    }
        
    pub fn center_rect(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical])
            .flex(Flex::Center)
            .areas(area);
        area
    }

    pub fn generate_block<'a>(name: String, title_color: Option<Color>, bg_color: Option<Color>) -> Block<'a> {
        Block::default()
            .title(name)
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_style(Style::default().fg(
                title_color.unwrap_or(Color::Rgb(0xff, 0xff, 0xff))
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
            .style(Style::default().bg(
                bg_color.unwrap_or(Color::Rgb(0x12, 0x12, 0x12))
            ))
    }
}

