use colored::Colorize;
use std::collections::HashSet;

use crate::utils::dir::get_current_dir;
use crate::utils::git::{get_git_status, get_git_output};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    border, DefaultTerminal, Frame, layout::{Alignment, Constraint, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, List, ListItem, ListState, Padding, Paragraph},
};


pub fn stage_files(mut files: Option<Vec<String>>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let staged_files_initial = get_staged_files()?;

    if all {
        let mut file = Vec::new();
        file.push(".".to_string());
        files = Some(file);
    }
    
    let files = match files {
        Some(x) => x,
        None => {
            println!("No files passed");
            return Ok(())
        }
    };
    
    for file in files.iter() {
        stage_file(file)?;
    }

    let staged_files_final = get_staged_files()?;
    let diff = list_diff(staged_files_final, staged_files_initial);
    for value in diff.into_iter() {
        println!("{} -> {}", value.red(), value.green());
    }
    Ok(())
}

fn stage_file(file: &String) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let _ = get_git_status(&vec!["add", &file], &current_dir)?;

    Ok(())
}

pub fn unstage(file: Option<String>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let staged_files_initial = get_staged_files()?;

    if file.is_none() && !all {
        return Err("No file specified".into());
    }

    if !all {
        let file = file.unwrap();
        unstage_file(file)?;
    } else {
        let current_dir = get_current_dir();

        let output = get_git_output(&vec!["status", "--porcelain"], &current_dir)?;

        let status = String::from_utf8_lossy(&output.stdout).to_string();
        for line in status.lines() {
            if line.starts_with("A ") || line.starts_with("M ") {
                unstage_file(line[3..].to_string())?;
            }
        }
    }
    
    let staged_files_final = get_staged_files()?;
    let diff = list_diff(staged_files_initial, staged_files_final);
    for value in diff.into_iter() {
        println!("{} -> {}", value.green(), value.red());
    }

    Ok(())
}

fn unstage_file(file: String) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();

    let status = get_git_status(&vec!["restore", "--staged", &file], &current_dir)?;

    if !status.success() {
        return Err(format!("Failed to stage file {}", file).into())
    }
  
    Ok(())
}

fn get_staged_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let current_dir = get_current_dir();
    let output = get_git_output(&vec!["diff", "--cached", "--name-only"], &current_dir)?;

    let staged_files_list = String::from_utf8_lossy(&output.stdout).to_string();
    let staged_files = string_to_vec(staged_files_list)?;
    Ok(staged_files)
}

fn string_to_vec(list: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result: Vec<String> = Vec::new();
    for line in list.lines() {
        result.push(line.trim().to_string());
    }
    return Ok(result)
}

fn list_diff(l1: Vec<String>, l2: Vec<String>) -> Vec<String> {
    let list_2: HashSet<String> = l2.into_iter().collect();

    l1.into_iter()
        .filter(|s| !list_2.contains(s))
        .clone()
        .collect()
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
enum State {
    New,
    Modified,
    Deleted,
    Renamed,
    Error,
}
struct StatefulCheckBox<'a> {
    files: Vec<String>,
    states: Vec<State>,
    added: Vec<bool>,
    original_states: Vec<bool>,
    list: List<'a>,
    selected: ListState,
}
impl StatefulCheckBox<'_> {
    fn new() -> Self {
        let (files, added, states) = Self::get_file_states();
        let list = Self::generateList(files.clone(), added.clone(), states.clone());
        Self {
            files: files,
            states: states,
            added: added.clone(),
            original_states: added,
            list: list, 
            selected: ListState::default().with_selected(Some(0)),
        }
    }
    fn get_file_states() -> (Vec<String>, Vec<bool>, Vec<State>) {
        let current_dir = get_current_dir();
        let args = vec!["status", "--porcelain"];
        let output = get_git_output(&args, &current_dir).unwrap();
        
        if !output.status.success() {
            let _stderr = String::from_utf8_lossy(&output.stderr);
            return (Vec::new(), Vec::new(), Vec::new());
        }
    
        let output = String::from_utf8_lossy(&output.stdout).to_string();
        let mut files: Vec<String> = Vec::new();
        let mut added: Vec<bool> = Vec::new();
        let mut states: Vec<State> = Vec::new();
        
        for line in output.lines() {
            match &line[..2] {
                "A " | "M " | "D " | "R "=> {
                    files.push(
                        line[3..]
                            .trim()
                            .strip_prefix('"')
                            .and_then(|f| f.strip_suffix('"'))
                            .unwrap_or(&line[3..])
                            .to_string()
                    );
                    added.push(true);
                    match &line[..2] {
                        "A " => states.push(State::New),
                        "M " => states.push(State::Modified),
                        "D " => states.push(State::Deleted),
                        "R " => states.push(State::Renamed),
                        _ => states.push(State::Error),
                    }
                },
                " M" | "??" | " D" | " R" => {
                    files.push(
                        line[3..]
                            .trim()
                            .strip_prefix('"')
                            .and_then(|f| f.strip_suffix('"'))
                            .unwrap_or(&line[3..])
                            .to_string()
                    );
                    added.push(false);
                    match &line[..2] {
                        "??" => states.push(State::New),
                        " M" => states.push(State::Modified),
                        " D" => states.push(State::Deleted),
                        " R" => states.push(State::Renamed),
                        _ => states.push(State::Error),
                    }
                },
                _ => {
                    files.push(
                        line[3..]
                            .trim()
                            .strip_prefix('"')
                            .and_then(|f| f.strip_suffix('"'))
                            .unwrap_or(&line[3..])
                            .to_string()
                    );
                    added.push(false);
                    states.push(State::Error);
                },
            };
        }
        return (files, added, states)
    }
    #[allow(nonstandard_style)]
    fn generateList(files: Vec<String>, added: Vec<bool>, states: Vec<State>) -> List<'static> {
        let mut list_items: Vec<ListItem> = Vec::new();
        for ((item, added_state), state) in files.iter().zip(added).zip(states) {
            let checkmark;
            let color;
            let print_state;
            match added_state {
                true => {
                    checkmark = "  ✅".to_string();
                    match state {
                        State::New => { 
                            color = Color::Green;
                            print_state = "      New    ";
                        },
                        State::Modified | State::Renamed => { 
                            color = Color::Yellow;
                            print_state = "      Mod    ";
                        },
                        State::Deleted => { 
                            color = Color::Red;
                            print_state = "      Del    ";
                        },
                        State::Error => { 
                            color = Color::LightRed;
                            print_state = "      Err    ";
                        },
                    }
                },
                false => {
                    checkmark = "  []".to_string();
                    match state {
                        State::New => { 
                            color = Color::Green;
                            print_state = "      New    ";
                        },
                        State::Modified | State::Renamed => { 
                            color = Color::Yellow;
                            print_state = "      Mod    ";
                        },
                        State::Deleted => { 
                            color = Color::Red;
                            print_state = "      Del    ";
                        },
                        State::Error => { 
                            color = Color::LightRed;
                            print_state = "      Err    ";
                        },
                    }
                }
            }
            list_items.push(ListItem::new(Line::from(vec![
                Span::styled(checkmark, Style::default()),
                Span::styled(print_state, Style::default().fg(color)),
                Span::styled(item.clone(), Style::default())
            ])));
        }

        List::new(list_items)
            .highlight_style(Style::new().gray().reversed())
            .block(
                Block::default().borders(border!(BOTTOM, LEFT, RIGHT))
            )
    }

    fn next(&mut self) {
        let i = match self.selected.selected() {
            Some(i) => {
                if i == self.files.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.selected.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len().saturating_sub(1)
                } else {
                    i.saturating_sub(1)
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }

    fn update(&mut self) {
        let i = match self.selected.selected() {
            Some(i) => i,
            None => return 
        };
        self.added[i] = !self.added[i];
        self.list = Self::generateList(self.files.clone(), self.added.clone(), self.states.clone())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
///// Rendering
/////////////////////////////////////////////////////////////////////////////////////////////////
pub fn stage_selector() -> Result<()> {
    let mut checkbox_list = StatefulCheckBox::new();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut checkbox_list);
    ratatui::restore();
    if result.unwrap() == true {
        get_file_changes(checkbox_list.files, checkbox_list.original_states, checkbox_list.added);
    }
    Ok(())
}

fn run(mut terminal: DefaultTerminal, checkbox_list: &mut StatefulCheckBox) -> Result<bool> {
    loop {
        terminal.draw(|frame| render(frame, checkbox_list))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Up => checkbox_list.previous(),
                KeyCode::Down => checkbox_list.next(),
                KeyCode::Enter => checkbox_list.update(),
                KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                _ => {}
            }
        }
    }
    Ok(true)
}

fn render(frame: &mut Frame, checkbox_list: &mut StatefulCheckBox) {
    render_interactive(frame, checkbox_list);
}

fn render_interactive(frame: &mut Frame, checkbox_list: &mut StatefulCheckBox) {
    let layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(3)
    ]).split(frame.area());

    render_header(frame, layout[0]);
    render_checkboxes(frame, layout[1], checkbox_list);
    render_footer(frame, layout[2]);
}
fn render_header(frame: &mut Frame, area: Rect) {
    let text = Paragraph::new("Staged   State   File Path")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(border!(TOP, LEFT, RIGHT))
        );
    frame.render_widget(text, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let text = Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Toggle  "),
            Span::styled("Ctrl+x", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Apply Changes  "),
            Span::styled("Esc/q", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Quit"),
        ]);

    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph.block(block), area);
}

fn render_checkboxes(frame: &mut Frame, area: Rect, checkbox_list: &mut StatefulCheckBox) {
    frame.render_stateful_widget(checkbox_list.list.clone(), area, &mut checkbox_list.selected);
}
/////////////////////////////////////////////////////////////////////////////////////////////////
///// Git Operations
/////////////////////////////////////////////////////////////////////////////////////////////////

fn get_file_changes(files: Vec<String>, old_states: Vec<bool>, new_states: Vec<bool>) {
    for ((old_state, new_state), filename) in old_states.iter().zip(&new_states).zip(&files) {
        if *old_state == false && *new_state == true {
            match stage_file(&filename) {
                Ok(_) => println!("{} -> {}", filename.red(), filename.green()),
                Err(_) => println!("Failed to stage {}", filename)
            }
        } else if *old_state == true && *new_state == false {
            match unstage_file(filename.clone()) {
                Ok(_) => println!("{} -> {}", filename.green(), filename.red()),
                Err(_) => println!("Failed to stage {}", filename)
            }
        }
    } 
}