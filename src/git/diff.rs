use crate::utils::dir::{get_current_dir, get_top_git};
use crate::utils::git::{FileState, get_git_output};

use crossterm::event::KeyEvent;
use crossterm::event::{self, Event::{self}, KeyCode, KeyModifiers};
use ratatui_crossterm::CrosstermBackend;
use ansi_to_tui::IntoText;
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Rect, Layout},
    Frame,
    style::{Modifier, Color, Style},
    widgets::{Block, Borders, List, ListState, Paragraph, Padding, Wrap},
    text::{Span,Text, Line},
};
use std::io::Stdout;
use std::process::Command;

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

        Self {
            list: List::new(items.clone())
                .style(Color::White)
                .highlight_style(Modifier::REVERSED)
                .block(
                    Block::bordered()
                        .title(title.clone())
                        .borders(Borders::ALL)),
            state: ListState::default().with_selected(Some(0)),
            items: items,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i.saturating_sub(1)
                }
            },
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
impl<'a> StatefulParagraph<'a> {
    fn new(text: Paragraph<'a>) -> Result<Self, Box<dyn std::error::Error>> {
        let p = text
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

    fn update(&mut self) {
        self.text = self.text
            .clone()
            .block(
                Block::bordered()
                    .title(Span::styled(self.title.clone(), self.default_style))
                    .title_bottom(Span::styled(self.subtitle.clone(), self.default_style))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
            );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
struct GitDiff<'a> {
    path: String,
    content: Paragraph<'a>,
    status: FileState,
    staged: bool,
}
impl<'a> GitDiff<'a> {
    fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {

        if file.len() < 2 { return Err("Empty git status --porcelain line".into()) }
        let status = match &file[..2] {
            "A " | " A" | "??" => FileState::New,
            "M " | " M" =>  FileState::Modified,
            "D " | " D" =>  FileState::Deleted,
            "R " | " R" =>  FileState::Renamed,
            _ =>  FileState::Error,
        };
        let staged = match &file[..1] {
            " " | "?" => false,
            "A" | "M" | "D" | "R" => true,
            _ => false
        };
        
        let mut content = Paragraph::new("");
        let mut path = file[3..].trim().to_string();

        if path.starts_with('"') && path.ends_with('"') {
            path = path[1..path.len()-1].to_string();
        }

        if status == FileState::Renamed {
            let (old, new) = path.split_once(" -> ").ok_or_else(|| { std::io::Error::new(std::io::ErrorKind::InvalidData, "Input does not contain ' -> '")})?;
            content = Paragraph::new(Line::from(vec![
                Span::styled(old.to_owned(), Style::default().fg(Color::Red)),
                Span::styled(" -> ", Style::default()),
                Span::styled(new.to_owned(), Style::default().fg(Color::Green))
            ]));
            path = new.to_string();
        }
        
        return Ok(Self {
            path: path,
            content: content,
            status: status,
            staged: staged,
        })
    }
    fn get_content(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_dir = get_current_dir();
        let top_dir = get_top_git()?;

        match self.status {
            FileState::New => {
                let file_path = format!("{}/{}", top_dir.display(), &self.path);

                let output = get_git_output(&vec!["diff", "-U1000000", "--raw", &self.path], &current_dir)?;
            
                if !output.status.success() {
                    let content = String::from_utf8_lossy(&output.stderr).to_string();
                    self.content = Paragraph::new(content);
                } else {
                    let content = std::fs::read_to_string(file_path)?;
                    self.content = Paragraph::new(Text::styled(content, Style::default().fg(Color::Green)));
                }
            }
            FileState::Modified => {
                let file_path = format!("{}/{}", top_dir.display(), &self.path);
 
                let args = match self.staged {
                    true => {
                        vec!["diff", "-U1000000", "--color=always", "HEAD^", "--", &file_path]
                    },
                    false => {
                        vec!["diff", "-U1000000", "--color=always", "HEAD^", "--", &file_path]
                    }
                };
                
                let output = Command::new("git")
                        .args(args)
                        .output()?;
                
                if !output.status.success() {
                    return Err(format!("git diff failed: {}", String::from_utf8_lossy(&output.stderr)).into());
                }

                let mut diff = String::from_utf8(output.stdout)?;
                diff = diff.lines().skip(5).collect::<Vec<&str>>().join("\n");
                let text: Text<'static> = diff.into_text()?;
                self.content = Paragraph::new(text);
            },
            FileState::Renamed => {},
            FileState::Deleted => {
                let head = format!("HEAD:{}", &self.path);
                let output = get_git_output(&vec!["show", &head], &current_dir)?;
    
                if !output.status.success() {
                    let content = String::from_utf8_lossy(&output.stderr).to_string();
                    self.content = Paragraph::new(content);
                } else {
                    let content = String::from_utf8_lossy(&output.stdout).to_string();
                    self.content = Paragraph::new(content).style(Style::default().fg(Color::Red));
                }
            }
            _ => {
                self.content = Paragraph::new("Error");
            }
       };
        Ok(())
    }
}

pub fn diff(file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["status", "--porcelain"], &current_dir)?;
    if !output.status.success() {
        return Err("'git status --porcelain' failed".into())
    }
    let mut files: Vec<GitDiff> = Vec::new();
    let output = String::from_utf8_lossy(&output.stdout).to_string();
    for line in output.lines() {
        let gitdiff = GitDiff::new(line)?;
        files.push(gitdiff);
    }

    let mut app = App::new(file, files)?;
    match app.run() {
        Ok(_) => app.exit(),
        Err(e) => {
            app.exit();
            return Err(e)
        }
    }
    Ok(())
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
    files: Vec<GitDiff<'a>>,
}
struct App<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    data: AppData<'a>,
}
impl<'a> App<'_> {
    fn new(file: Option<String>, files: Vec<GitDiff<'a>>) -> Result<App<'a>, Box<dyn std::error::Error>> {
        color_eyre::install()?;
        let terminal = ratatui::init();

        let paragraph = if let Some(f) = &file {
            let top_dir = get_top_git()?;
            let file_path = format!("{}/{}", top_dir.display(), f.trim());

            let current_dir = get_current_dir();
            let output = get_git_output(&vec!["status", "--porcelain", &file_path], &current_dir)?;
            if !output.status.success() {
                return Err("'git status --porcelain' failed".into())
            }
            let output = String::from_utf8_lossy(&output.stdout).to_string();
            let mut diff = GitDiff::new(&output)?;
            diff.get_content()?;
            StatefulParagraph::new(diff.content)?
        } else {
            StatefulParagraph::new(Paragraph::new(String::new()))?
        };
        
        let file_list = StatefulList::new(
            files.clone()
                 .into_iter()
                 .map(|file|file.path).collect());
        Ok(App {
            terminal: terminal,
            data: AppData { file: file, paragraph: paragraph, file_list: file_list, files: files }
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
                        app.files[selected_idx].get_content()?;
                        app.paragraph = StatefulParagraph::new(app.files[selected_idx].content.clone())?;
                        app.paragraph.update_title(selected_item);
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
        let layout = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3)
        ]).split(frame.area());
        
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(chunk_size),
                Constraint::Min(0)
            ])
            .split(layout[0]);

        if app.file.is_none() {
            Self::render_list(frame, app, chunks[0]);
            Self::render_diff(frame, app, chunks[1]);
        } else {
            Self::render_diff(frame, app, chunks[0]);
        }
        Self::render_footer(frame, layout[1]);
    }

    fn render_list(frame: &mut Frame, app: &mut AppData, chunk: Rect) {
        frame.render_stateful_widget(app.file_list.list.clone(), chunk, &mut app.file_list.state);
    }
    
    fn render_diff(frame: &mut Frame, app: &mut AppData, chunk: Rect) {
        app.paragraph.text = app.paragraph.text.clone().scroll((app.paragraph.scroll_offset, 0));
        frame.render_widget(app.paragraph.text.clone(), chunk);
    }

    fn render_footer(frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1));

        let text = Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Cyan).bold()),
                Span::raw(": Navigate Menu "),
                Span::styled("Enter", Style::default().fg(Color::Cyan).bold()),
                Span::raw(": View file  "),
                Span::styled("Ctrl+↑/↓", Style::default().fg(Color::Cyan).bold()),
                Span::raw(": Navigate file  "),
                Span::styled("Esc/q", Style::default().fg(Color::Cyan).bold()),
                Span::raw(": Quit"),
            ]);

        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph.block(block), area);
    }
    fn exit(&mut self) {
        ratatui::restore();
    }
}
