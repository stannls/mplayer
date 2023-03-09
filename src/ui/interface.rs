use crate::ui::components;
use std::{io::Stdout, sync::mpsc::Receiver};
use std::io;
use tui::{
    layout::{Constraint, Direction, Layout}, Terminal,
    backend::CrosstermBackend,
};
use crossterm::event::{KeyCode, KeyEvent, DisableMouseCapture};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;
use super::input::Event;

struct UiState{
    searching: bool,
    searchbar_content: String,
}

impl UiState {
    fn new() -> UiState{
        UiState { searching: false, searchbar_content: String::from("") }
    }
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?; 
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error>{
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?; 
    Ok(())
}

pub fn render_interface(terminal: &mut Terminal<CrosstermBackend<Stdout>>, rx: Receiver<Event<KeyEvent>>){
    let mut ui_state = UiState::new();
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let below_search_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(30), Constraint::Min(2)].as_ref())
                .split(main_layout[1]);
            

            f.render_widget(components::build_window_border(), size);
            f.render_widget(components::build_searchbar(ui_state.searching, &ui_state.searchbar_content), main_layout[0]);
            f.render_widget(components::build_main_window(), below_search_layout[1]);
            f.render_widget(components::build_side_menu(), below_search_layout[0])
        }).unwrap();
        if !ui_state.searching {
            match rx.recv().unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('s') => {
                        ui_state.searching = true;
                    }
                    _ => {}
                },
                Event::Tick => {}
            }
        } else {
            match rx.recv().unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char(c) => ui_state.searchbar_content.push(c),
                    KeyCode::Backspace => {
                        ui_state.searchbar_content.pop();
                    }
                    KeyCode::Esc => {
                        ui_state.searching = false;
                        ui_state.searchbar_content.clear();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

}
