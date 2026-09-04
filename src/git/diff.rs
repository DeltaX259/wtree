use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_output};

use crossterm::event::{self, Event::{self}, KeyCode, KeyModifiers};
use color_eyre::Result;
use regex::Regex;
use ansi_to_tui::IntoText;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Rect, Layout},
    Frame,
    style::{Modifier, Color, Style},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    text::Span,
};

struct StatefulList<'a> {
    list: List<'a>,
    state: ListState,
    items: Vec<String>,
}
impl StatefulList<'_> {
    fn new(items: Vec<String>) -> Self {
        let default_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);
        
        let title = Span::styled("Changed files", default_style);
        let subtitle = Span::styled(
            " Scroll: Up/Down     View file: Enter     Quit: q/Esc ",
            default_style);

        Self {
            list: List::new(items.clone())
                .style(Color::White)
                .highlight_style(Modifier::REVERSED)
                .block(
                    Block::bordered()
                        .title(title.clone())
                        .title_bottom(subtitle.clone())
                        .borders(Borders::ALL)),
            state: ListState::default().with_selected(Some(0)),
            items: items,

        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1).min(self.items.len().saturating_sub(1)),
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[derive(Default)]
pub struct StatefulParagraph<'a> {
    text: Paragraph<'a>,
    scroll_offset: u16,
    max_scroll: u16,
    title: String,
    subtitle: String,
    default_style: Style,
}
impl StatefulParagraph<'_> {
    fn new(text: String) -> Self {
        let t2 = text.into_text().unwrap();
        let p = Paragraph::new(t2)
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title(Span::styled("", Style::default().add_modifier(Modifier::BOLD)))
                    .title_bottom(Span::styled("", Style::default().add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
            );
        
        Self {
            text: p,
            scroll_offset: 0,
            max_scroll: 0,
            title: String::new(),
            subtitle: String::new(),
            default_style: Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        }
    }
    fn next(&mut self) {
        if self.scroll_offset < self.max_scroll {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }
    
    fn previous(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
    
    fn update_title(&mut self, title: String) {
        self.title = title;
        self.update();
    }
    
    fn update_subtitle(&mut self, subtitle: String) {
        self.subtitle = subtitle;
        self.update();
    }

    fn update(&mut self) {
        self.text = self.text
            .clone()
            .block(
                Block::bordered()
                    .title(Span::styled(self.title.clone(), self.default_style))
                    .title_bottom(Span::styled(self.subtitle.clone(), self.default_style))
                    .borders(Borders::ALL)
            );
    }
}

fn get_diff(file: &str) -> Result<String, Box<dyn std::error::Error>> {
    if file == String::from("") {
        return Ok(String::from(""));
    }
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["diff", "-U1000000", "--word-diff", &file], &current_dir)?;

    let mut diff_lines = String::from_utf8_lossy(&output.stdout).to_string();
    diff_lines = skip_lines(&diff_lines, 5);

    let regex_added = Regex::new(r"\{([+])|([+])\}").expect("Invalid regex");
    let regex_removed = Regex::new(r"\[([-])|([-])\]").expect("Invalid regex");
    let reset = "\x1b[0m";

    diff_lines = parse_diff(&diff_lines, regex_added, "\x1b[1;30;42m", reset); // Green
    diff_lines = parse_diff(&diff_lines, regex_removed, "\x1b[1;30;41m", reset); // Red

    Ok(diff_lines)
}

fn skip_lines(s: &str, n: usize) -> String {
    s.lines()
        .skip(n)
        .collect::<Vec<&str>>()
        .join("\n")
}

fn parse_diff(text: &str, reg: Regex, r1: &str, r2: &str) -> String {
    reg.replace_all(&text, |caps: &regex::Captures| {
        match caps.get(2) {
            Some(_) => r2,
            None => r1,
        }
    }).to_string()
}

pub fn diff(file: Option<String>) -> Result<(), Box< dyn std::error::Error>> {
    let files = get_list();
    let mut stateful_files = StatefulList::new(files);

    color_eyre::install()?;
    ratatui::run(|terminal| {
        let _ = app(terminal, &mut stateful_files, file);
    });

    Ok(())
}

fn get_list() -> Vec<String> {
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["diff", "--name-only"], &current_dir).expect("Failed to run git diff --name-only");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToString::to_string)
        .collect()
}

fn app(terminal: &mut DefaultTerminal, mut file_list: &mut StatefulList, file: Option<String>) -> Result<(), Box<dyn std::error::Error>>{
    let mut p1 = if let Some(f) = &file {
        let content = get_diff(&f)?;
        let mut p = StatefulParagraph::new(content);
        p.update_title(f.clone());
        p.update_subtitle(" Scroll: Up/Down Quit: q/Esc ".to_string());
        p
    } else {
        StatefulParagraph::new(String::new())
    };

    loop {
        let area = terminal.size()?;
        p1.max_scroll = std::cmp::max(area.height, (p1.text.line_count(area.width / 2) as u16) + 3);
        p1.max_scroll -= area.height;
            
        terminal.draw(|frame| {
            render(frame, &mut file_list, &mut p1, &file);
        })?;
        
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Down | KeyCode::Char('s') => {
                    if key.modifiers == KeyModifiers::CONTROL || !file.is_none() {
                        p1.next()
                    } else {
                        file_list.next()
                    }
                },
                KeyCode::Up | KeyCode::Char('w') => {
                    if key.modifiers == KeyModifiers::CONTROL || !file.is_none() {
                        p1.previous();
                    } else {
                        file_list.previous()
                    }
                },
                KeyCode::Enter => {
                    if file.is_none() {
                        if let Some(selected_idx) = file_list.state.selected() {
                            let selected_item = file_list.items[selected_idx].clone();
                            let content = get_diff(&selected_item)?;
                            p1 = StatefulParagraph::new(content);
                            p1.update_title(selected_item);
                            p1.update_subtitle(" Scroll: Ctrl+Up/Down ".to_string());
                        }
                    }
                }
                _ => {},
            }
        }
    }
}

fn render(frame: &mut Frame, file_list: &mut StatefulList, p1: &mut StatefulParagraph, file: &Option<String>) {

    let chunk_size: u16 ;
    if file.is_none() {
        chunk_size = 50;
    } else {
        chunk_size = 100;
    }
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(chunk_size),
            Constraint::Min(0)
        ])
        .split(frame.area());
    
    if file.is_none() {
        render_list(frame, file_list, chunks[0]);
        render_diff(frame, p1, chunks[1]);
    } else {
        render_diff(frame, p1, chunks[0]);
    }
}

fn render_list(frame: &mut Frame, file_list: &mut StatefulList, chunk: Rect) {
    frame.render_stateful_widget(file_list.list.clone(), chunk, &mut file_list.state);
}

fn render_diff(frame: &mut Frame, p1: &mut StatefulParagraph, chunk: Rect) {
    p1.text = p1.text.clone().scroll((p1.scroll_offset, 0));
    frame.render_widget(p1.text.clone(), chunk);
}