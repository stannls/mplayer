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
    pub(crate) focused_result: FocusedResult,
}


#[derive(Clone)]
pub(crate) enum MainWindowState {
    Welcome,
    Results((Vec<Recording>, Vec<Artist>, Vec<Release>)),
    SongFocus(Release),
    ArtistFocus(Artist),
    RecordFocus(Recording)
}

#[derive(Clone)]
pub(crate) enum FocusedResult {
    None,
    Song(usize),
    Record(usize),
    Artist(usize),
    Playlist(usize)
}

impl UiState {
    fn new() -> UiState{
        UiState { searching: false, searchbar_content: String::from(""), quit: false, main_window_state: MainWindowState::Welcome, focused_result: FocusedResult::None }
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
                MainWindowState::Welcome => f.render_widget(components::build_main_window(), content_layout[1]),
                MainWindowState::SongFocus(_) => f.render_widget(components::build_song_focus(), content_layout[1]),
                MainWindowState::RecordFocus(_) => f.render_widget(components::build_record_focus(), content_layout[1]),
                MainWindowState::ArtistFocus(_) => f.render_widget(components::build_artist_focus(), content_layout[1]),
                MainWindowState::Results(t) => {
                    let scroll_value = match ui_state.focused_result {
                        FocusedResult::Song(t) | FocusedResult::Record(t) | FocusedResult::Artist(t) | FocusedResult::Playlist(t) => Some(t),
                        _ => None
                    };
                    let result_layout = layout::build_search_layout(content_layout[1]);
                    let displayable_results = result_layout[0].height as usize - 3;
                    f.render_widget(components::build_result_box(String::from("[S]ong"), t.2, if matches!(ui_state.focused_result, FocusedResult::Song(_)) {scroll_value} else {None}, displayable_results), result_layout[0]);
                    f.render_widget(components::build_result_box(String::from("[A]rtist"), t.1, if matches!(ui_state.focused_result, FocusedResult::Artist(_)) {scroll_value} else {None}, displayable_results), result_layout[1]);
                    f.render_widget(components::build_result_box(String::from("[R]ecord"), t.0, if matches!(ui_state.focused_result, FocusedResult::Record(_)) {scroll_value} else {None}, displayable_results), result_layout[2]);
                    f.render_widget(components::build_result_box::<wrapper::Artist>(String::from("[P]laylist"), vec![], if matches!(ui_state.focused_result, FocusedResult::Playlist(_)) {scroll_value} else {None}, displayable_results), result_layout[3]);
            },
            _ => {}
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
            KeyCode::Char('A') if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Artist(_)) => ui_state.focused_result = FocusedResult::Artist(0),
            KeyCode::Char('S') if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Song(_)) => ui_state.focused_result = FocusedResult::Song(0),
            KeyCode::Char('R') if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Record(_)) => ui_state.focused_result  = FocusedResult::Record(0),
            KeyCode::Char('P') if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Playlist(_)) => ui_state.focused_result = FocusedResult::Playlist(0),
            KeyCode::Down => match ui_state.focused_result {
                FocusedResult::Song(t) => if helpers::check_scroll_space_down(ui_state) {ui_state.focused_result = FocusedResult::Song(t+1)},
                FocusedResult::Record(t) => if helpers::check_scroll_space_down(ui_state) {ui_state.focused_result = FocusedResult::Record(t+1)},
                FocusedResult::Artist(t) => if helpers::check_scroll_space_down(ui_state) {ui_state.focused_result = FocusedResult::Artist(t+1)},
                _ => {}
            },
            KeyCode::Up => match ui_state.focused_result {
                FocusedResult::Song(t) => if t>0 {ui_state.focused_result = FocusedResult::Song(t-1)},
                FocusedResult::Record(t) => if t>0 {ui_state.focused_result = FocusedResult::Record(t-1)},
                FocusedResult::Artist(t) => if t>0 {ui_state.focused_result = FocusedResult::Artist(t-1)},
                _ => {}
            },
            KeyCode::Enter => {
                match ui_state.main_window_state.clone() { 
                    MainWindowState::Results(r) => {
                       match ui_state.focused_result {
                           FocusedResult::Song(id) => {
                               ui_state.focused_result = FocusedResult::None;
                               ui_state.main_window_state = MainWindowState::SongFocus(r.2.get(id).unwrap().clone());    
                           },
                           FocusedResult::Record(id) => {
                                ui_state.focused_result = FocusedResult::None;
                                ui_state.main_window_state = MainWindowState::RecordFocus(r.0.get(id).unwrap().clone());
                           },
                           FocusedResult::Artist(id) => {
                               ui_state.focused_result = FocusedResult::None;
                               ui_state.main_window_state = MainWindowState::ArtistFocus(r.1.get(id).unwrap().clone());
                           },
                           _ => {}
                       } 
                    },
                    _ => {}
                }
            }
            _ => {}

       } 
    }
}
