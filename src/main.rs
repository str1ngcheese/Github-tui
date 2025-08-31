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
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, ListState},
};

#[derive(Debug)]
enum AppState {
    Listing,
    Viewing,
}

#[derive(Debug)]
struct App {
    state: AppState,
    dotfiles: Vec<PathBuf>,
    list_state: ListState,
}

impl App {
    fn new() -> io::Result<Self> {
        let dotfiles = find_dotfiles()?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Self {
            state: AppState::Listing,
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
            let area = frame.size();
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
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, area, &mut app.list_state);
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
