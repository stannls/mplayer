use super::components::ToolbarType;
use super::input::Event;
use super::input::handle_input;
use crate::api::Artist;
use crate::api::{Song, Album};
use crate::api::download::download_pool::DownloadPool;
use crate::api::download::musify_downloader::MusifyDownloader;
use crate::api::fs::scan_artists;
use crate::ui::{components, layout};
use crossterm::event::{DisableMouseCapture, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use std::io;
use std::{io::Stdout, sync::mpsc::Receiver};
use tui::{backend::CrosstermBackend, Terminal};


use crate::api::search::wrapper::{self, ArtistWrapper, SongWrapper, ReleaseGroupWrapper};

#[derive(Clone)]
pub(crate) struct UiState {
    pub(crate) searching: bool,
    pub(crate) searchbar_content: String,
    pub(crate) quit: bool,
    pub(crate) main_window_state: MainWindowState,
    pub(crate) focused_result: FocusedResult,
    pub(crate) last_search: Option<(Vec<ReleaseGroupWrapper>, Vec<ArtistWrapper>, Vec<SongWrapper>)>,
    pub(crate) artists: Vec<String>,
}

#[derive(Clone)]
pub(crate) enum MainWindowState {
    Welcome,
    Results((Vec<ReleaseGroupWrapper>, Vec<ArtistWrapper>, Vec<SongWrapper>)),
    SongFocus(Box<dyn Song>),
    ArtistFocus(Box<dyn Artist>, Option<usize>),
    RecordFocus(Box<dyn Album>, Option<usize>),
}

#[derive(Clone)]
pub(crate) enum FocusedResult {
    None,
    Song(usize),
    Record(usize),
    Artist(usize),
    Playlist(usize),
    Libary(usize),
}

impl UiState {
    fn new() -> UiState {
        UiState {
            searching: false,
            searchbar_content: String::from(""),
            quit: false,
            main_window_state: MainWindowState::Welcome,
            focused_result: FocusedResult::None,
            last_search: None,
            artists: vec![],
        }
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

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn render_interface(terminal: &mut Terminal<CrosstermBackend<Stdout>>,rx: Receiver<Event<KeyEvent>>) {
    // Init for ui state and the downloader
    let mut ui_state = UiState::new();
    let downloader = DownloadPool::new(4)
        .add_downloader(MusifyDownloader::new());

    // Main UI render loop
    while !ui_state.quit {
        terminal
            .draw(|f| {

                // Layouting
                let size = f.size();
                let main_layout = layout::build_main_layout().split(size);
                let content_layout = layout::build_content_layout().split(main_layout[1]);
                let focus_layout = layout::build_focus_layout().split(content_layout[1]);
                let result_layout = layout::build_search_layout(content_layout[1]);

                // Main window border
                f.render_widget(components::build_window_border(), size);

                // Searchbar
                let search = if ui_state.searching {Some(ui_state.searchbar_content.to_owned())} else {None};
                f.render_widget(components::build_searchbar(search), main_layout[0]);

                // Side menu
                let index = match ui_state.focused_result {
                    FocusedResult::Libary(i) => Some(i),
                    _ => None,
                };
                f.render_widget(components::build_side_menu(ui_state.artists.to_owned(), index, content_layout[0].height as usize - 3), content_layout[0]);

                // The main content window
                match ui_state.main_window_state.to_owned() {
                    // The default welcome screen
                    MainWindowState::Welcome => {
                        f.render_widget(components::build_welcome_window(), content_layout[1])
                    }
                    // The window for viewing details to a song
                    MainWindowState::SongFocus(s) => {
                        f.render_widget(components::build_song_focus(s.to_owned()), content_layout[1]);
                        if s.is_local() {
                            f.render_widget(components::build_focus_toolbox(ToolbarType::Play), focus_layout[1]);
                        } else {
                            f.render_widget(components::build_focus_toolbox(ToolbarType::Download), focus_layout[1]);
                        }
                    }
                    // The window for viewing details to a record
                    MainWindowState::RecordFocus(r, index) => {
                        f.render_widget(components::build_record_focus(r.to_owned(), index, content_layout[1].height as usize - 3), content_layout[1]);
                        if r.is_local(){
                            f.render_widget(components::build_focus_toolbox(ToolbarType::Play), focus_layout[1]);
                        } else{
                            f.render_widget(components::build_focus_toolbox(ToolbarType::Download), focus_layout[1]);
                        }
                    }
                    // The window for viewing details to an artist
                    MainWindowState::ArtistFocus(a, index) => {
                        f.render_widget(components::build_artist_focus(a, index, content_layout[1].height as usize - 3), content_layout[1]);
                        f.render_widget(components::build_focus_toolbox(ToolbarType::Default), focus_layout[1]);
                    }
                    MainWindowState::Results(t) => {
                        // Determines which of the search results is focused
                        let scroll_value = match ui_state.focused_result {
                            FocusedResult::Song(t) => (Some(t), None, None, None),
                            FocusedResult::Record(t) => (None, Some(t), None, None),
                            FocusedResult::Artist(t) => (None, None, Some(t), None),
                            FocusedResult::Playlist(t) => (None, None, None, Some(t)),
                            _ => (None, None, None, None),
                        };
                        // Calculates how many results can be rendered on the current screen
                        let displayable_results = result_layout[0].height as usize - 3;
                        // Song search results
                        f.render_widget(components::build_result_box("[S]ong".to_string(), t.2, scroll_value.0, displayable_results), result_layout[0]);
                        // Record search results
                        f.render_widget(components::build_result_box("[R]ecord".to_string(), t.0, scroll_value.1, displayable_results), result_layout[2]);
                        // Artist search results
                        f.render_widget(components::build_result_box("[A]rtist".to_string(), t.1, scroll_value.2, displayable_results), result_layout[1]);
                        // Playlsist search results (Not implemented)
                        f.render_widget(components::build_result_box::<wrapper::ArtistWrapper>("[P]laylist".to_string(), vec![], scroll_value.3, displayable_results),result_layout[3]);
                    }
                }
            })
        .unwrap();

        // Handles keyboard input
        match rx.recv().unwrap() {
            Event::Input(event) => handle_input(event, &mut ui_state, &downloader).await,
            _ => {}
        }
        ui_state.artists = scan_artists();
    }
}


