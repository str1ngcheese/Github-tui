use std::{
    io::{self, stdout, Error, ErrorKind, Stdout},
    path::PathBuf,
};
use walkdir::WalkDir;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph},
};

#[derive(Debug)]
struct App {
    dotfiles: Vec<PathBuf>,
    list_state: ListState,
}

impl App {
    fn new() -> io::Result<Self> {
        let dotfiles = find_dotfiles()?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Self {
            dotfiles,
            list_state,
        })
    }
}

fn main() -> io::Result<()> {
    // Setup the terminal
    let mut terminal = init_terminal()?;

    // Create the app
    let mut app = App::new()?;

    // Main application loop
    let result = run(&mut terminal, &mut app);

    // Restore the terminal
    restore_terminal()?;

    result
}

// Setup the terminal
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

// Main application loop
fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let list_items: Vec<ListItem> = app
                .dotfiles
                .iter()
                .map(|path| ListItem::new(path.to_string_lossy().to_string()))
                .collect();

            let list = List::new(list_items)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Gray),
                )
                .highlight_symbol(">> ")
                .block(Block::default().title("Dotfiles").borders(Borders::ALL));

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(frame.size());

            frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let selected_path = if let Some(selected) = app.list_state.selected() {
                app.dotfiles.get(selected)
            } else {
                None
            };

            let preview_content = if let Some(path) = selected_path {
                if path.is_dir() {
                    "This is a directory.".to_string()
                } else {
                    std::fs::read_to_string(path)
                        .unwrap_or_else(|_| "Error reading file.".to_string())
                }
            } else {
                "No file selected.".to_string()
            };

            let preview = Paragraph::new(preview_content).block(
                Block::default()
                    .title("Preview")
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1)),
            );

            frame.render_widget(preview, chunks[1]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if let Some(selected) = app.list_state.selected() {
                            if selected < app.dotfiles.len() - 1 {
                                app.list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = app.list_state.selected() {
                            if selected > 0 {
                                app.list_state.select(Some(selected - 1));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn find_dotfiles() -> io::Result<Vec<PathBuf>> {
    let home_dir = std::env::var("HOME").map_err(|e| Error::new(ErrorKind::NotFound, e))?;
    let config_dir_path = PathBuf::from(format!("{}/.config", home_dir));

    // 1. Find dotfiles/dot-directories in the HOME directory (shallow)
    let mut home_dotfiles: Vec<PathBuf> = WalkDir::new(&home_dir)
        .max_depth(1)
        .min_depth(1) // Exclude the home directory itself
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
        .map(|entry| entry.into_path())
        .collect();

    // 2. Find all files and directories inside .config (recursive)
    // We also exclude the .config directory itself from the list
    let mut config_files: Vec<PathBuf> = WalkDir::new(&config_dir_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .collect();

    // 3. Combine the lists
    home_dotfiles.append(&mut config_files);

    Ok(home_dotfiles)
}

// Restore the terminal
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
