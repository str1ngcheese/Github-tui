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

fn main() -> io::Result<()> {
    // Setup the terminal
    let mut terminal = init_terminal()?;

    // Main application loop
    let result = run(&mut terminal);

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
fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let dotfiles = find_dotfiles()?;
    let list_items: Vec<ListItem> = dotfiles
        .iter()
        .map(|path| ListItem::new(path.to_string_lossy().to_string()))
        .collect();

    let mut selected_index = 0;
    let mut list_state = ListState::default();

    loop {
        // This line connects the number 'selected_index' to the UI state
        list_state.select(Some(selected_index));

        terminal.draw(|frame| {
            let area = frame.size();
            let list = List::new(list_items.clone())
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Gray),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, area, &mut list_state);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if selected_index < dotfiles.len() - 1 {
                            selected_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected_index > 0 {
                            selected_index -= 1;
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
