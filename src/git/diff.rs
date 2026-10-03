use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_output};

use crossterm::event::KeyEvent;
use crossterm::event::{self, Event::{self}, KeyCode, KeyModifiers};
use ratatui_crossterm::CrosstermBackend;
use color_eyre::Result;
use regex::Regex;
use ansi_to_tui::IntoText;
use ratatui::{
    Terminal,
    layout::{Constraint, Rect, Layout},
    Frame,
    style::{Modifier, Color, Style},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    text::Span,
};
use std::io::Stdout;

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
struct StatefulParagraph<'a> {
    text: Paragraph<'a>,
    scroll_offset: u16,
    max_scroll: u16,
    title: String,
    subtitle: String,
    default_style: Style,
}
impl StatefulParagraph<'_> {
    fn new(text: String) -> Result<Self, Box<dyn std::error::Error>> {
        let t2 = text.into_text()?;
        let p = Paragraph::new(t2)
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title(Span::styled("", Style::default().add_modifier(Modifier::BOLD)))
                    .title_bottom(Span::styled("", Style::default().add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
            );
        
        Ok(Self {
            text: p,
            scroll_offset: 0,
            max_scroll: 0,
            title: String::new(),
            subtitle: String::new(),
            default_style: Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        })
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
    println!("current dir {}", current_dir.display());
    println!("file: {}", &file);
    let output = get_git_output(&vec!["diff", "-U1000000", "--word-diff", &file], &current_dir)?;

    let mut diff_lines = String::from_utf8_lossy(&output.stdout).to_string();
    println!("content: {}", &diff_lines);
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
    let stateful_files = StatefulList::new(files);

    let mut app = App::new(file, stateful_files)?;
    app.run()?;
    app.exit();

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
////////////////////////////////////////////////////////////////////////////////////////////////
enum ButtonAction {
    View,
    Exit, 
    Nothing,
}

struct AppData<'a> {
    file: Option<String>,
    paragraph: StatefulParagraph<'a>,
    file_list: StatefulList<'a>,
}
struct App<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    data: AppData<'a>,
}
impl<'a> App<'_> {
    fn new(file: Option<String>, file_list: StatefulList<'a>) -> Result<App<'a>, Box<dyn std::error::Error>> {
        color_eyre::install()?;
        let terminal = ratatui::init();

        let paragraph = if let Some(f) = &file {
            let content = get_diff(&f)?;
            let mut p = StatefulParagraph::new(content)?;
            p.update_title(f.clone());
            p.update_subtitle(" Scroll: Up/Down Quit: q/Esc ".to_string());
            p
        } else {
            StatefulParagraph::new(String::new())?
        };

        Ok(App {
            terminal: terminal,
            data: AppData { file: file, paragraph: paragraph, file_list: file_list }
        })
    }
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let area = self.terminal.size()?;
            self.data.paragraph.max_scroll = std::cmp::max(area.height, (self.data.paragraph.text.line_count(area.width / 2) as u16) + 3);
            self.data.paragraph.max_scroll -= area.height;

            self.terminal.draw(|frame| {
                Self::render(frame, &mut self.data);
            })?;

            if let Event::Key(key) = event::read()? {
                match Self::button_pressed(&mut self.data, key)? {
                    ButtonAction::View => {},
                    ButtonAction::Nothing => {},
                    ButtonAction::Exit => { 
                        return Ok(())
                   },
                }
            }
        }
    }
    fn button_pressed(app: &mut AppData, key: KeyEvent) -> Result<ButtonAction, Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(ButtonAction::Exit),
            KeyCode::Down | KeyCode::Char('s') => {
                if key.modifiers == KeyModifiers::CONTROL || !app.file.is_none() {
                    app.paragraph.next();
                    return Ok(ButtonAction::Nothing)
                } else {
                    app.file_list.next();
                    return Ok(ButtonAction::Nothing)
                }
            },
            KeyCode::Up | KeyCode::Char('w') => {
                if key.modifiers == KeyModifiers::CONTROL || !app.file.is_none() {
                    app.paragraph.previous();
                    return Ok(ButtonAction::Nothing)

                } else {
                    app.file_list.previous();
                    return Ok(ButtonAction::Nothing)
                }
            },
            KeyCode::Enter => {
                if app.file.is_none() {
                    if let Some(selected_idx) = app.file_list.state.selected() {
                        let selected_item = app.file_list.items[selected_idx].clone();
                        let content = get_diff(&selected_item)?;
                        app.paragraph = StatefulParagraph::new(content)?;
                        app.paragraph.update_title(selected_item);
                        app.paragraph.update_subtitle(" Scroll: Ctrl+Up/Down ".to_string());
                        return Ok(ButtonAction::View)
                    }
                }
                return Ok(ButtonAction::Nothing)
            }
            _ => return Ok(ButtonAction::Nothing),
        }
    }
    fn render(frame: &mut Frame, app: &mut AppData) {

        
        let chunk_size: u16 ;
        if app.file.is_none() {
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

        if app.file.is_none() {
            Self::render_list(frame, app, chunks[0]);
            Self::render_diff(frame, app, chunks[1]);
        } else {
            Self::render_diff(frame, app, chunks[0]);
        }
    }

    fn render_list(frame: &mut Frame, app: &mut AppData, chunk: Rect) {
        frame.render_stateful_widget(app.file_list.list.clone(), chunk, &mut app.file_list.state);
    }
    
    fn render_diff(frame: &mut Frame, app: &mut AppData, chunk: Rect) {
        app.paragraph.text = app.paragraph.text.clone().scroll((app.paragraph.scroll_offset, 0));
        frame.render_widget(app.paragraph.text.clone(), chunk);
    }
    fn exit(&mut self) {
        ratatui::restore();
    }
}
