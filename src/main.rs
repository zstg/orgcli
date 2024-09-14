#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
use std::env;
use std::fs::File;
use std::io::{self, Read};
use ratatui::{
    prelude::Stylize,
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    DefaultTerminal,
};
fn main() -> io::Result<()> {
    // Initialize terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;

    // Get file path from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file-path>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];

    // Load Org mode file contents
    let file_contents = read_file_contents(file_path)?;
    let file_lines: Vec<&str> = file_contents.lines().collect();

    // Run application
    let app_result = run(terminal, file_lines);
    ratatui::restore();
    app_result
}

fn read_file_contents(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn run(mut terminal: DefaultTerminal, file_lines: Vec<&str>) -> io::Result<()> {
    let mut line_start = 0;
    let mut terminal_size = terminal.size()?;
    let mut lines_per_page = terminal_size.height as usize;

    loop {
        // Update terminal size
        terminal_size = terminal.size()?;
        lines_per_page = terminal_size.height as usize;
        
        // Ensure lines_per_page is at least 1
        if lines_per_page == 0 {
            lines_per_page = 1;
        }

        // Determine the range of lines to display
        let end = std::cmp::min(line_start + lines_per_page, file_lines.len());
        let mut formatted_lines = Vec::new();

        for line in &file_lines[line_start..end] {
            formatted_lines.push(format_line(line));
        }

        terminal.draw(|frame| {
            let paragraph = Paragraph::new(formatted_lines.clone())
                .style(Style::default().fg(ratatui::style::Color::White).bg(ratatui::style::Color::Black));
            frame.render_widget(paragraph, frame.area());
        })?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()), // Quit
                KeyCode::Down | KeyCode::Char('j') => {
                    if line_start + 1 < file_lines.len() {
                        line_start += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if line_start > 0 {
                        line_start -= 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn format_line(line: &str) -> Line<'_> {
    let mut spans = Vec::new();

    if line.starts_with("***") {
        // Header 3, normal size, bold
        spans.push(Span::styled(&line[3..], Style::default().add_modifier(Modifier::BOLD)));
    } else if line.starts_with("**") {
        // Header 2, smaller than header 1, bold
        spans.push(Span::styled(&line[2..], Style::default().add_modifier(Modifier::BOLD)));
    } else if line.starts_with("*") {
        // Header 1, larger font, bold
        spans.push(Span::styled(&line[1..], Style::default().add_modifier(Modifier::BOLD)));
    } else {
        // Handle inline formatting like _word_ (underline) and /word/ (italic)
        let mut remaining = line;
        while let Some(start) = remaining.find(|c| c == '_' || c == '/') {
            if remaining.chars().nth(start).unwrap() == '_' {
                let end = remaining[start + 1..].find('_').unwrap_or(remaining.len() - 1) + start + 1;
                spans.push(Span::raw(&remaining[..start]));
                spans.push(Span::styled(&remaining[start + 1..end], Style::default().add_modifier(Modifier::UNDERLINED)));
                remaining = &remaining[end + 1..];
            } else if remaining.chars().nth(start).unwrap() == '/' {
                let end = remaining[start + 1..].find('/').unwrap_or(remaining.len() - 1) + start + 1;
                spans.push(Span::raw(&remaining[..start]));
                spans.push(Span::styled(&remaining[start + 1..end], Style::default().add_modifier(Modifier::ITALIC)));
                remaining = &remaining[end + 1..];
            }
        }
        spans.push(Span::raw(remaining));
    }
    Line::from(spans)
}
