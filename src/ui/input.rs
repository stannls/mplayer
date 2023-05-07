use crossterm::event::{self, KeyEvent, KeyCode};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
use crate::api::fs::FsArtist;
use crate::api::player::MusicPlayer;
use crate::api::search::remote::{unique_releases, album_from_release_group_id};
use crate::api::search::wrapper::AlbumWrapper;
use crate::ui::helpers;
use super::interface::{UiState, MainWindowState, FocusedResult};
use crate::api::download::download_pool::DownloadPool;


pub enum Event<I> {
    Input(I),
    Tick,
}

pub fn create_input_channel() -> Receiver<Event<KeyEvent>> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let crossterm::event::Event::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });
    rx
}

pub(crate) async fn handle_input(input: KeyEvent, ui_state: &mut UiState, downloader: &DownloadPool, music_player: &MusicPlayer) {
    // Match arm for inputting text
    if ui_state.searching {
        handle_search_input(input, ui_state).await
    } else {
        // Match arm for everything else
        match input.code {
            KeyCode::Char('d') => match ui_state.main_window_state.to_owned() {
                MainWindowState::SongFocus(s) => if !s.is_local() {
                    downloader.download_song(s)
                },
                MainWindowState::RecordFocus(r, _) => if !r.is_local(){
                    downloader.download_songs(r.get_songs())
                },
                _ => {}
            },
            KeyCode::Char('p') => match ui_state.main_window_state.to_owned() {
                MainWindowState::SongFocus(s) => if s.is_local() {music_player.play_song(s)},
                MainWindowState::RecordFocus(r, _) => if r.is_local() {music_player.play_album(r)},
                _ => {}
            },
            KeyCode::Char(' ') => music_player.pause(),
            KeyCode::Char('n') => music_player.skip(),
            KeyCode::Char('v') => music_player.stop(),
            KeyCode::Char('q') => ui_state.quit = true,
            KeyCode::Char('s') => {
                ui_state.searching = true;
                ui_state.focused_result = FocusedResult::None;
            },
            KeyCode::Char('A') => {
                if matches!(ui_state.main_window_state, MainWindowState::Results(_))
                    && !matches!(ui_state.focused_result, FocusedResult::Artist(_))
                    {
                        ui_state.focused_result = FocusedResult::Artist(0)
                    }
            }
            KeyCode::Char('S') => {
                if matches!(ui_state.main_window_state, MainWindowState::Results(_))
                    && !matches!(ui_state.focused_result, FocusedResult::Song(_))
                    {
                        ui_state.focused_result = FocusedResult::Song(0)
                    }
            }
            KeyCode::Char('R') => {
                if matches!(ui_state.main_window_state, MainWindowState::Results(_))
                    && !matches!(ui_state.focused_result, FocusedResult::Record(_))
                    {
                        ui_state.focused_result = FocusedResult::Record(0)
                    }
            }
            KeyCode::Char('P') => {
                if matches!(ui_state.main_window_state, MainWindowState::Results(_))
                    && !matches!(ui_state.focused_result, FocusedResult::Playlist(_))
                    {
                        ui_state.focused_result = FocusedResult::Playlist(0)
                    }
            },
            KeyCode::Char('L') => {
                if !matches!(ui_state.focused_result, FocusedResult::Libary(_)) {
                    ui_state.focused_result = FocusedResult::Libary(0);
                    match ui_state.main_window_state.to_owned() {
                        MainWindowState::RecordFocus(r, _) => ui_state.main_window_state = MainWindowState::RecordFocus(r, None),
                        MainWindowState::ArtistFocus(a, _) => ui_state.main_window_state = MainWindowState::ArtistFocus(a, None),
                        _ => {}
                    }
                }
            }
            KeyCode::Down => {
                match ui_state.focused_result {
                    FocusedResult::Libary(index) => {
                        if ui_state.artists.len() - index > 1{
                            ui_state.focused_result = FocusedResult::Libary(index+1);
                        }
                    },
                    _ => {}
                }
                match ui_state.main_window_state.clone(){
                    MainWindowState::Results(_) => match ui_state.focused_result {
                        FocusedResult::Song(t) => {
                            if helpers::check_scroll_space_down(ui_state) {
                                ui_state.focused_result = FocusedResult::Song(t + 1)
                            }
                        }
                        FocusedResult::Record(t) => {
                            if helpers::check_scroll_space_down(ui_state) {
                                ui_state.focused_result = FocusedResult::Record(t + 1)
                            }
                        }
                        FocusedResult::Artist(t) => {
                            if helpers::check_scroll_space_down(ui_state) {
                                ui_state.focused_result = FocusedResult::Artist(t + 1)
                            }
                        },
                        _ => {}

                    },
                    MainWindowState::ArtistFocus(a, index) => {
                        if !matches!(ui_state.focused_result, FocusedResult::Libary(_)){
                            if index.is_none() {
                                ui_state.main_window_state = MainWindowState::ArtistFocus(a, Some(0));
                            } else if a.get_albums().len() - index.unwrap() > 1 {
                                ui_state.main_window_state = MainWindowState::ArtistFocus(a, Some(index.unwrap()+1));
                            }

                        }
                    },
                    MainWindowState::RecordFocus(r, index) => {

                        if !matches!(ui_state.focused_result, FocusedResult::Libary(_)){
                            if index.is_none() {
                                ui_state.main_window_state = MainWindowState::RecordFocus(r, Some(0));
                            } else if r.get_songs().len() - index.unwrap() > 1 {
                                ui_state.main_window_state = MainWindowState::RecordFocus(r, Some(index.unwrap()+1));
                            }
                        }

                    },
                    _ => {}

                };
            },
            KeyCode::Up => {
                match ui_state.main_window_state.clone(){
                    MainWindowState::Results(_) => match ui_state.focused_result {
                        FocusedResult::Song(t) => {
                            if t > 0 {
                                ui_state.focused_result = FocusedResult::Song(t - 1)
                            }
                        }
                        FocusedResult::Record(t) => {
                            if t > 0 {
                                ui_state.focused_result = FocusedResult::Record(t - 1)
                            }
                        }
                        FocusedResult::Artist(t) => {
                            if t > 0 {
                                ui_state.focused_result = FocusedResult::Artist(t - 1)
                            }
                        }
                        _ => {}
                    },
                    MainWindowState::ArtistFocus(a, index) => if !matches!(ui_state.focused_result, FocusedResult::Libary(_)) && index.is_some() && index.unwrap() > 0 {ui_state.main_window_state = MainWindowState::ArtistFocus(a, Some(index.unwrap()-1))},
                    MainWindowState::RecordFocus(r, index) => if !matches!(ui_state.focused_result, FocusedResult::Libary(_)) && index.is_some() && index.unwrap() > 0 {ui_state.main_window_state = MainWindowState::RecordFocus(r, Some(index.unwrap()-1))},
                    _ => {}

                };
                match ui_state.focused_result{
                    FocusedResult::Libary(index) => {
                        if index > 0 {
                            ui_state.focused_result = FocusedResult::Libary(index-1);
                        }
                    },
                    _ => {}
                }
            },
            KeyCode::Char('b') => match ui_state.main_window_state {
                MainWindowState::SongFocus(_)
                    | MainWindowState::ArtistFocus(_, _)
                    | MainWindowState::RecordFocus(_, _) => {
                        if matches!(ui_state.last_search, Some(_)) {
                            ui_state.main_window_state =
                                MainWindowState::Results(ui_state.last_search.clone().unwrap());
                            ui_state.last_search = None;
                        }
                    }
                _ => {}
            },
            KeyCode::Enter => {
                match ui_state.main_window_state.clone() {
                    MainWindowState::Results(r) => match ui_state.focused_result {
                        FocusedResult::Song(id) => {
                            ui_state.focused_result = FocusedResult::None;
                            ui_state.last_search = Some(r.clone());
                            ui_state.main_window_state =
                                MainWindowState::SongFocus(Box::new(r.2.get(id).unwrap().clone()));
                        }
                        FocusedResult::Record(id) => {
                            ui_state.focused_result = FocusedResult::None;
                            ui_state.last_search = Some(r.clone());
                            ui_state.main_window_state = {
                                MainWindowState::RecordFocus(
                                    Box::new(AlbumWrapper::new(album_from_release_group_id(&r.0.get(id).unwrap().data.id).await)), None)
                            }
                        }
                        FocusedResult::Artist(id) => {
                            ui_state.focused_result = FocusedResult::None;
                            ui_state.last_search = Some(r.clone());
                            let albums = unique_releases(r.1.get(id).unwrap().clone().data.id).await;
                            ui_state.main_window_state = MainWindowState::ArtistFocus(Box::new(r.1.get(id).unwrap().releases(albums)), None);
                        }
                        _ => {}
                    },
                    MainWindowState::ArtistFocus(a, index) => if index.is_some() {
                        ui_state.main_window_state =
                            if a.get_albums().get(index.unwrap()).unwrap().is_groups() {
                                MainWindowState::RecordFocus(Box::new(AlbumWrapper::new(album_from_release_group_id(&a.get_albums().get(index.unwrap()).unwrap().get_id()).await)), None)
                            } else {
                                MainWindowState::RecordFocus(a.get_albums().get(index.unwrap()).unwrap().to_owned(), None)};

                    },
                    MainWindowState::RecordFocus(r, index) => if index.is_some() {
                        ui_state.main_window_state = MainWindowState::SongFocus(r.get_songs().get(index.unwrap()).unwrap().to_owned());
                    }

                    _ => {}
                };
                match ui_state.focused_result {
                    FocusedResult::Libary(index) => {
                        ui_state.focused_result = FocusedResult::None;
                        ui_state.main_window_state = MainWindowState::ArtistFocus(Box::new(FsArtist::new(ui_state.artists.get(index).unwrap().to_owned()).unwrap()), None);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}

async fn handle_search_input(input: KeyEvent, ui_state: &mut UiState) {
    match input.code {
        KeyCode::Char(c) => ui_state.searchbar_content.push(c),
        KeyCode::Backspace => {
            ui_state.searchbar_content.pop();
        }
        KeyCode::Esc => {
            ui_state.searching = false;
            ui_state.searchbar_content.clear();
        }
        KeyCode::Enter => helpers::query_web(ui_state).await,
        _ => {}
    }

}
