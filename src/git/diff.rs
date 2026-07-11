use std::process::Command;
use crate::utils::dir::get_current_dir;
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Frame, layout::{Layout}};
use color_eyre::Result;
use regex::Regex;
use ansi_to_tui::{IntoText};

pub fn diff(file: String) -> Result<(), Box< dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = Command::new("git")
        .args(["diff", "-U1000000", "--word-diff", &file])
        .current_dir(current_dir)
        .output()?;
    
    let diff_lines = String::from_utf8_lossy(&output.stdout).to_string();
    let diff_lines = skip_lines(&diff_lines, 5);

    let re_added = Regex::new(r"\{([+])|([+])\}").expect("Invalid regex");
    let re_subbed = Regex::new(r"\[([-])|([-])\]").expect("Invalid regex");

    let mut r1 = "\x1b[1;42m";
    let r2 = "\x1b[0m";
    let mut text = parse_diff(&diff_lines, re_added, r1, r2);
    r1 = "\x1b[1;41m";
    text = parse_diff(&text, re_subbed, r1, r2);
    let t2 = text.into_text().unwrap();

    let p = Paragraph::new(t2)        
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("New")
                .borders(Borders::ALL));

    
    color_eyre::install()?;
    ratatui::run(|terminal| {
        let _ = app(terminal, p);
    });

    Ok(())
}

fn skip_lines(s: &str, n: usize) -> String {
    s.lines()
        .skip(n)
        .collect::<Vec<&str>>()
        .join("\n")
}

fn parse_diff(text: &str, reg: Regex, r1: &str, r2: &str) -> String {
    let text = reg.replace_all(&text, |caps: &regex::Captures| {
            match caps.get(2) {
                Some(_) => r2,
                None => r1,
            }
        }).to_string();
    text
}

fn app(terminal: &mut DefaultTerminal, m1: Paragraph) -> Result<(), Box<dyn std::error::Error>>{
    let mut scroll_offset: u16 = 0;
    loop {
        let area = terminal.size()?;
        let max_scroll = area.height.saturating_add(3);

        terminal.draw(|frame| {
            render(frame, m1.clone(), scroll_offset);
        })?;
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c')) {
                break Ok(());
            } else if key.code == KeyCode::Up {
                scroll_offset = scroll_offset.saturating_sub(1);
            } else if key.code == KeyCode::Down && scroll_offset <= max_scroll {
                scroll_offset = scroll_offset.saturating_add(1);
            }
        }
    }
}

fn render(frame: &mut Frame, m1: Paragraph, scroll_offset: u16) {
     let chunks = Layout::default()
         .direction(ratatui::layout::Direction::Horizontal)
         .constraints([
             Constraint::Percentage(50),
             Constraint::Min(0)
         ])
         .split(frame.area());
    let p1 = m1.scroll((scroll_offset, 0));
    
    frame.render_widget(p1, chunks[0]);
}