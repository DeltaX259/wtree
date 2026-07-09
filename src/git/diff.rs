use std::process::Command;
use crate::utils::dir::get_current_dir;
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::macros::ratatui_core::terminal;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use crossterm::event::{self, Event, KeyEvent, KeyCode};
use ratatui::{Frame, layout::{Alignment, Layout}, style::Stylize};
use color_eyre::Result;

#[derive(Debug)]
enum DiffLine {
    Context(String),
    Unchanged(String),
    Added(String),
    Removed(String),
    Header(String),
}

pub fn diff(file: String) -> Result<(), Box< dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = Command::new("git")
        .args(["diff", &file])
        .current_dir(current_dir)
        .output()?;
    let diff_lines = String::from_utf8_lossy(&output.stdout).to_string();
    let text = parse_diff(&diff_lines);
    let mut lines: Vec<Line<'static>> = Vec::new();
    for line in text.iter() {
        match line {
            DiffLine::Added(s) => lines.push(Line::from(Span::styled(
                format!("+{}", s),
                Style::default().fg(Color::Green)
            ))),
            DiffLine::Removed(s) => lines.push(Line::from(Span::styled(
                format!("-{}", s),
                Style::default().fg(Color::Red),
            ))),
            DiffLine::Unchanged(s) => lines.push(Line::from(
                s.clone()
            )),
            _ => continue
        };
    }
    
    color_eyre::install()?;
    ratatui::run(|terminal| {
        let _ = app(terminal, lines);
    });

    Ok(())
}

fn parse_diff(diff: &str) -> Vec<DiffLine> {
    diff.lines()
        .map(|line| {
            if line.starts_with("@@") {
                DiffLine::Header(line.into())
            } else if line.starts_with('+') && !line.starts_with("+++") {
                DiffLine::Added(line[1..].into())
            } else if line.starts_with('-') && !line.starts_with("---") {
                DiffLine::Removed(line[1..].into())
            } else if line.starts_with(' ') {
                DiffLine::Unchanged(line[1..].into())
            } else {
                DiffLine::Context(line.into())
            }
        }).collect()
}


fn app(terminal: &mut DefaultTerminal, message: Vec<Line>) -> Result<(), Box<dyn std::error::Error>>{
    loop {
        terminal.draw(|frame| {
            render(frame, message.clone());
        })?;
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break Ok(());
            }
        }
    }
}
fn render(frame: &mut Frame, message: Vec<Line>) {
     let chunks = Layout::default()
         .direction(ratatui::layout::Direction::Horizontal)
         .constraints([
             Constraint::Length(80),
             Constraint::Min(0)
         ])
         .split(frame.area());
     
    let paragraph = Paragraph::new(message);
    //         .scroll((scroll_offset, 0));
    frame.render_widget(paragraph, chunks[0]);
}