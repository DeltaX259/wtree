use std::path::PathBuf;
use crate::utils::dir::{
    get_current_dir,
    get_top_dir,
};
use crate::utils::worktree::get_current_worktree;
use crate::utils::git::{get_git_status, get_git_output};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use ratatui_core::layout::{Constraint, Direction, Layout};
use ratatui_core::text::Line;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::terminal::{Terminal, Frame};
use ratatui_crossterm::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui_crossterm::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui_crossterm::{CrosstermBackend, crossterm};
use ratatui_textarea::TextArea;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::Borders;
use std::io;
use std::io::{StdoutLock};
use std::process::Command;


pub fn amend(all: bool, push: bool) -> Result<(), Box<dyn std::error::Error>> {
    let branch = get_current_worktree()?;
    let path = get_top_dir()?;
    let current_branch = PathBuf::from(format!("{}/{}", path.display(), branch).trim().to_string());

    if all {
        let _ = get_git_status(&vec!["add", "."], &current_branch)?;
    }

    let _ = get_git_status(&vec!["commit", "--amend", "--no-edit"], &current_branch)?;

    if push {
        git_push(true)?;
    }
    
    Ok(())
}

pub fn git_push(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let branch = get_current_worktree()?;

    let mut args = vec!("push", "-q");

    match check_upstream() {
        Err(_) => {
            args.push("--set-upstream");
            args.push("origin");
            args.push(&branch);
            println!("Set upstream branch to: {}", &branch);
        },
        _ => (),
    }

    if force {
        args.push("--force-with-lease");
    }

    let _ = get_git_status(&args, &current_dir)?;

    println!("Push successful");

    Ok(())
}

fn check_upstream() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"], &current_dir)?;

    if output.status.success() {
        return Ok(())
    } else {
        return Err("No upstream found".into())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
struct AppData<'a> {
    textarea: Vec<TextBox<'a>>,
    layout: Layout,
    footer: Line<'a>,
    which: usize,
}
struct App<'a> {
    terminal:  Terminal<CrosstermBackend<StdoutLock<'a>>>,
    data: AppData<'a>,
}
impl<'a> App<'_> {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut stdout = io::stdout().lock();
        enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let textarea = vec![
            TextBox::new("Commit title", Active::Active),
            TextBox::new("Commit message", Active::NotActive),
        ];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(90), Constraint::Min(1)].as_ref());

        let footer = ratatui::text::Line::from("Tab = Switch textbox | Ctrl+x = Finish commit | Esc = Quit").centered().style(Style::new().blue());
        let which = 0;

        Ok(Self {
            terminal: terminal,
            data: AppData { textarea, layout, footer, which },
        })
    }
    fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            let terminal = &mut self.terminal;
            terminal.draw(|frame| {
                Self::render(frame, &mut self.data)
            })?;
            if let Event::Key(key) = event::read()? {
                match Self::button_pressed(&mut self.data, key) {
                    Some(save_changes) => {
                        return Ok(save_changes)
                    },
                    None => {},
                }
            }
        }
    }
    fn button_pressed(app: &mut AppData, key: KeyEvent) -> Option<bool>{
        match key.code {
            KeyCode::Esc => {
                return Some(false)
            },
            KeyCode::Tab => {
                app.textarea[0].switch();
                app.which = (app.which + 1) % 2;
                app.textarea[1].switch();
                return None
            },
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(true)
            }
            _ => {
                app.textarea[app.which].textarea.input(key);
                return None
            }
        }
    }
    fn render(frame: &mut Frame, app: &mut AppData) {
        let chunks = app.layout.split(frame.area());
        for (textarea, chunk) in app.textarea.iter().zip(chunks.iter()) {
            frame.render_widget(&textarea.textarea, *chunk);
        }
        frame.render_widget(&app.footer, chunks[2]);
    }
    fn exit(&mut self, save_state: bool) -> Result<(), Box<dyn std::error::Error>> {
        exit(&mut self.terminal)?;
        if save_state {
            git_commit(&self.data.textarea)?;
        }
        return Ok(())
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn make_commit() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    let result = app.run();
    match result {
        Ok(save_state) => {
            app.exit(save_state)?;
            return Ok(())
        }
        Err(e) => return Err(e)
    }
}

enum Active {
    Active,
    NotActive,
}

struct TextBox<'a> {
    textarea: TextArea<'a>,
    title: String,
    active: Active,
}
impl TextBox<'_> {
    fn new(text: &str, active: Active) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());

        match active {
            Active::Active => {
                textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .title(text.to_owned()),
                );
            }
            Active::NotActive => {
                textarea.set_cursor_style(Style::default());
                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::DarkGray))
                        .title(text.to_owned()),
                );
            }
        }
        Self {
            textarea: textarea,
            title: text.to_owned(),
            active: active,
        }
    }

    fn switch(&mut self) {
        self.active = match self.active {
            Active::Active => {
                self.textarea.set_cursor_style(Style::default());
                self.textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::DarkGray))
                        .title(self.title.to_owned()),
                );
                Active::NotActive
            }
            Active::NotActive => {
                self.textarea
                    .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                self.textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .title(self.title.to_owned()),
                );
                Active::Active
            }
        }
    }
}

fn exit(term: &mut Terminal<CrosstermBackend<std::io::StdoutLock<'_>>>) -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

fn git_commit(textarea: &Vec<TextBox>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let status = Command::new("git")
        .args(["commit", "-m", textarea[0].textarea.lines().join("\n").as_str(), "-m", textarea[1].textarea.lines().join("\n").as_str(), "-q"])
        .current_dir(current_dir)
        .status()?;
    if status.success() {
        println!("Commit successful");
    } else {
        println!("Commit failed");
    }

    Ok(())
}