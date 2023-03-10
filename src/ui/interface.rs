use crate::ui::{components, layout, helpers};
use crate::api::search::wrapper::{Artist, Recording, Release, self};
use std::{io::Stdout, sync::mpsc::Receiver};
use std::io;
use tui::{
    Terminal,
    backend::CrosstermBackend,
};
use crossterm::event::{KeyCode, KeyEvent, DisableMouseCapture};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, LeaveAlternateScreen};
use crossterm::execute;
use super::input::Event;

#[derive(Clone)]
pub(crate) struct UiState{
    pub(crate) searching: bool,
    pub(crate) searchbar_content: String,
    quit: bool,
    pub(crate) main_window_state: MainWindowState,
}


#[derive(Clone)]
pub(crate) enum MainWindowState {
    Welcome,
    Results((Vec<Recording>, Vec<Artist>, Vec<Release>))
}

impl UiState {
    fn new() -> UiState{
        UiState { searching: false, searchbar_content: String::from(""), quit: false, main_window_state: MainWindowState::Welcome }
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

pub async fn render_interface(terminal: &mut Terminal<CrosstermBackend<Stdout>>, rx: Receiver<Event<KeyEvent>>){
    let mut ui_state = UiState::new();
    while !ui_state.quit {
        terminal.draw(|f| {
            let size = f.size();

            let main_layout = layout::build_main_layout().split(size);
            let content_layout = layout::build_content_layout().split(main_layout[1]);
            
            f.render_widget(components::build_window_border(), size);
            f.render_widget(components::build_searchbar(&ui_state.searching, &ui_state.searchbar_content), main_layout[0]);
            f.render_widget(components::build_side_menu(), content_layout[0]);

            match ui_state.clone().main_window_state {
                crate::ui::interface::MainWindowState::Welcome => f.render_widget(components::build_main_window(), content_layout[1]),
                crate::ui::interface::MainWindowState::Results(t) => {
                    let result_layout = layout::build_search_layout(content_layout[1]);
                    f.render_widget(components::build_result_box(String::from("Song"), t.2), result_layout[0]);
                    f.render_widget(components::build_result_box(String::from("Artist"), t.1), result_layout[1]);
                    f.render_widget(components::build_result_box(String::from("Record"), t.0), result_layout[2]);
                    f.render_widget(components::build_result_box::<wrapper::Artist>(String::from("Playlist"), vec![]), result_layout[3]);
            }
        }
        }).unwrap();

        // Handles keyboard input
        match rx.recv().unwrap() {
            Event::Input(event) => handle_input(event, &mut ui_state).await,
            _ => {}
        } 
    }

}

async fn handle_input(input: KeyEvent, ui_state: &mut UiState){
    // Match arm for inputting text
    if ui_state.searching{
        match input.code {
            KeyCode::Char(c) => ui_state.searchbar_content.push(c),
            KeyCode::Backspace => {
                        ui_state.searchbar_content.pop();
            }
            KeyCode::Esc => {
                ui_state.searching = false;
                ui_state.searchbar_content.clear();
            },
            KeyCode::Enter => helpers::query_web(ui_state).await,
            _ => {}

        }
    // Match arm for everything else
    } else {
        match input.code {
            KeyCode::Char('q') => ui_state.quit = true,
            KeyCode::Char('s') => ui_state.searching = true,
            _ => {}

       } 
    }
}
