use std::env::current_dir;
use std::rc::Rc;
use std::path::PathBuf;
use std::process::Command;
use crate::utils::dir::get_current_dir;
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Rect};
use crossterm::event::{self, Event::{self, Key}, KeyCode, KeyModifiers};
use ratatui::{Frame, layout::{Layout}};
use ratatui::style::{Modifier, Color};
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph, Wrap};
use color_eyre::Result;
use regex::Regex;
use ansi_to_tui::{IntoText};


struct StatefulList {
    state: ListState,
    items: Vec<String>,
}
impl StatefulList {
    fn new(items: Vec<String>) -> Self {
        Self {
            state: ListState::default().with_selected(Some(0)),
            items,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        // println!("Next: {:?}",  self.state.selected());
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        // println!("Previous: {:?}",  self.state.selected());
    }
}


fn get_diff(file: String) -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = Command::new("git")
        .args(["diff", "-U1000000", "--word-diff", &file])
        .current_dir(current_dir)
        .output()?;
    
    let mut diff_lines = String::from_utf8_lossy(&output.stdout).to_string();
    diff_lines = skip_lines(&diff_lines, 5);

    let regex_added = Regex::new(r"\{([+])|([+])\}").expect("Invalid regex");
    let regex_removed = Regex::new(r"\[([-])|([-])\]").expect("Invalid regex");

    let regex1 = "\x1b[1;42m";
    let regex2 = "\x1b[0m";

    diff_lines = parse_diff(&diff_lines, regex_added, regex1, regex2);

    let regex1 = "\x1b[1;41m";
    diff_lines = parse_diff(&diff_lines, regex_removed, regex1, regex2);
    Ok(diff_lines)
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

pub fn diff2(file: String) -> Result<(), Box< dyn std::error::Error>> {
    let files = get_list();
    let mut stateful_files = StatefulList::new(files);

    color_eyre::install()?;
    ratatui::run(|terminal| {
        let _ = app(terminal, &mut stateful_files);
    });

    Ok(())
}

fn get_list() -> Vec<String> {
    let current_dir = get_current_dir();
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(current_dir)
        .output().unwrap();

    let f = String::from_utf8_lossy(&output.stdout).to_string();
    let mut files: Vec<String> = Vec::new();
    for line in f.lines() {
        files.push(line.to_string());
    }
    files
}

fn app(terminal: &mut DefaultTerminal, file_list: &mut StatefulList) -> Result<(), Box<dyn std::error::Error>>{
    // let mut list_state = ListState::default().with_selected(Some(0));
    let mut content = String::from("");
    // let frame = terminal.get_frame();

    
    loop {
        let mut list_state = file_list.state;
        terminal.draw(|frame| {
            render(frame, &file_list, &mut list_state, &content);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Down => {
                    file_list.next()
                },
                KeyCode::Up => {
                    file_list.previous()
                },
                KeyCode::Enter => {
                    if let Some(selected_idx) = file_list.state.selected() {
                        let selected_item = file_list.items[selected_idx].clone();
                        content = get_diff(selected_item).unwrap();
                    }
                }
                _ => {},
            }
        }
    }
}

fn render(frame: &mut Frame, file_list: &StatefulList, list_state: &mut ListState, content: &str) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Min(0)
        ])
        .split(frame.area());
     render_list(frame, file_list, list_state, chunks[0]);
     render_diff(frame, content.to_string(), chunks[1]);
}

fn render_list(frame: &mut Frame, file_list: &StatefulList, list_state: &mut ListState, chunk: Rect) {
    let list = List::new(file_list.items.clone())
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ")
        .block(
            Block::bordered()
                .title("New")
                .borders(Borders::ALL));
        
    frame.render_stateful_widget(list, chunk, list_state);
}

fn render_diff(frame: &mut Frame, string_display: String, chunk: Rect) {
    let p1 = Paragraph::new(string_display.into_text().unwrap())
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("New")
                .borders(Borders::ALL)
        );
    frame.render_widget(p1, chunk);
}