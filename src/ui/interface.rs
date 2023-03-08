use std::{io::Stdout, sync::mpsc::Receiver};
use std::io;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph}, Terminal,
    backend::CrosstermBackend,
};
use crossterm::event::{KeyCode, KeyEvent, DisableMouseCapture};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;
use super::input::Event;

enum State {
    Default,
    Input,
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
    let mut current_state = State::Default;
    let mut search = String::from("");
    loop {
        terminal.draw(|f| {
            let size = f.size();

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let block = Block::default().title("mplayer").borders(Borders::ALL);
            f.render_widget(block, size);

            let searchbar_content = if matches!(current_state, State::Default) {
                "Enter Search Term [s]"
            } else {
                search.as_str()
            };
            let searchbar = Paragraph::new(searchbar_content)
                .style(Style::default().add_modifier(Modifier::ITALIC))
                .block(Block::default().borders(Borders::ALL).title("Search"));
            let below_search_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(30), Constraint::Min(2)].as_ref())
                .split(main_layout[1]);
            let side_menu = Paragraph::new("Albums")
                .block(Block::default().borders(Borders::ALL).title("Libary"));

            let main_content = Paragraph::new("Lorem ipsum dolor sit amet.")
                .block(Block::default().borders(Borders::ALL).title("Welcome"));
            f.render_widget(searchbar, main_layout[0]);
            f.render_widget(main_content, below_search_layout[1]);
            f.render_widget(side_menu, below_search_layout[0])
        }).unwrap();
        if matches!(current_state, State::Default) {
            match rx.recv().unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('s') => {
                        current_state = State::Input;
                    }
                    _ => {}
                },
                Event::Tick => {}
            }
        } else {
            match rx.recv().unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char(c) => search.push(c),
                    KeyCode::Backspace => {
                        search.pop();
                    }
                    KeyCode::Esc => {
                        current_state = State::Default;
                        search = String::from("");
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

}
