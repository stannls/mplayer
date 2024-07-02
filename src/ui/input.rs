use crossterm::event::{self, KeyCode, KeyEvent};

use super::interface::{Focus, FocusedResult, MainWindowState, SideMenu, UiState};
use crate::api::fs::MusicRepository;
use crate::api::player::MusicPlayer;
use crate::ui::helpers;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use std::thread;

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

pub(crate) async fn handle_input(
    input: KeyEvent,
    ui_state: &mut UiState,
    music_player: &MusicPlayer,
    music_repository: &mut MusicRepository,
) {
    // Match arm for inputting text
    if ui_state.searching {
        handle_search_input(input, ui_state).await
    } else {
        if !ui_state.delete {
            // Match arm for everything else
            match input.code {
                KeyCode::Char('p') => match ui_state.main_window_state.to_owned() {
                    MainWindowState::SongFocus(s) => {
                        if s.is_local() {
                            music_player.play_song(s, true)
                        }
                    }
                    MainWindowState::RecordFocus(r, _) => {
                        if r.is_local() {
                            music_player.play_album(r, true)
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('e') => match ui_state.main_window_state.to_owned() {
                    MainWindowState::SongFocus(s) => {
                        if s.is_local() {
                            music_player.play_song(s, false)
                        }
                    }
                    MainWindowState::RecordFocus(r, _) => {
                        if r.is_local() {
                            music_player.play_album(r, false)
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('D') => match ui_state.main_window_state.clone() {
                    MainWindowState::SongFocus(s) => {
                        if s.is_local() {
                            ui_state.delete = true;
                        }
                    }
                    MainWindowState::RecordFocus(r, _) => {
                        if r.is_local() {
                            ui_state.delete = true;
                        }
                    }
                    MainWindowState::ArtistFocus(a, _) => {
                        if a.is_local() {
                            ui_state.delete = true;
                        }
                    }
                    _ => {}
                },
                KeyCode::Char(' ') => music_player.pause(),
                KeyCode::Char('n') => music_player.skip(),
                KeyCode::Char('v') => music_player.stop(),
                KeyCode::Char('q') => ui_state.quit = true,
                KeyCode::Char('+') => music_player.change_volume(0.1),
                KeyCode::Char('-') => music_player.change_volume(-0.1),
                KeyCode::Char('h') => {
                    ui_state.main_window_state = MainWindowState::Help;
                    ui_state.focus = Focus::None
                }
                KeyCode::Char('c') => {
                    let song_info = music_player.get_song_info();
                    if song_info.is_some() {
                        let current_album =
                            music_repository.find_current_album(&song_info.unwrap());
                        if current_album.is_some() {
                            ui_state.main_window_state =
                                MainWindowState::RecordFocus(current_album.unwrap(), None);
                        }
                    }
                }
                KeyCode::Char('s') => {
                    ui_state.searching = true;
                    ui_state.focused_result = FocusedResult::None;
                }
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
                }
                KeyCode::Char('L') => {
                    if !matches!(ui_state.side_menu, SideMenu::Libary(_)) {
                        ui_state.side_menu = SideMenu::Libary(None);
                        ui_state.focus = Focus::SideWindow;
                        match ui_state.main_window_state.to_owned() {
                            MainWindowState::RecordFocus(r, _) => {
                                ui_state.main_window_state = MainWindowState::RecordFocus(r, None)
                            }
                            MainWindowState::ArtistFocus(a, _) => {
                                ui_state.main_window_state = MainWindowState::ArtistFocus(a, None)
                            }
                            _ => {}
                        }
                    } else {
                        ui_state.focus = Focus::SideWindow;
                    }
                }
                KeyCode::Char('Q') => {
                    if !matches!(ui_state.side_menu, SideMenu::Queue(_)) {
                        ui_state.side_menu = SideMenu::Queue(None);
                        ui_state.focus = Focus::SideWindow;
                        match ui_state.main_window_state.to_owned() {
                            MainWindowState::RecordFocus(r, _) => {
                                ui_state.main_window_state = MainWindowState::RecordFocus(r, None)
                            }
                            MainWindowState::ArtistFocus(a, _) => {
                                ui_state.main_window_state = MainWindowState::ArtistFocus(a, None)
                            }
                            _ => {}
                        }
                    } else {
                        ui_state.focus = Focus::SideWindow;
                    }
                }
                KeyCode::Down => match ui_state.focus {
                    Focus::MainWindow => match ui_state.main_window_state.clone() {
                        MainWindowState::Results(_) => match ui_state.focused_result {
                            FocusedResult::Song(i) => {
                                if helpers::check_scroll_space_down(ui_state) {
                                    ui_state.focused_result = FocusedResult::Song(i + 1)
                                }
                            }
                            FocusedResult::Record(i) => {
                                if helpers::check_scroll_space_down(ui_state) {
                                    ui_state.focused_result = FocusedResult::Record(i + 1)
                                }
                            }
                            FocusedResult::Artist(i) => {
                                if helpers::check_scroll_space_down(ui_state) {
                                    ui_state.focused_result = FocusedResult::Artist(i + 1)
                                }
                            }
                            _ => {}
                        },
                        MainWindowState::ArtistFocus(a, i) => {
                            if a.get_albums().len() - i.unwrap_or(0) > 0 {
                                if i.is_some() {
                                    ui_state.main_window_state =
                                        MainWindowState::ArtistFocus(a, Some(i.unwrap() + 1))
                                } else {
                                    ui_state.main_window_state =
                                        MainWindowState::ArtistFocus(a, Some(0))
                                }
                            }
                        }
                        MainWindowState::RecordFocus(r, i) => {
                            if r.get_songs().len() - i.unwrap_or(0) > 0 {
                                if i.is_some() {
                                    ui_state.main_window_state =
                                        MainWindowState::RecordFocus(r, Some(i.unwrap() + 1))
                                } else {
                                    ui_state.main_window_state =
                                        MainWindowState::RecordFocus(r, Some(0))
                                }
                            }
                        }
                        _ => {}
                    },
                    Focus::SideWindow => match ui_state.side_menu {
                        SideMenu::Libary(i) => {
                            if ui_state.artists.len() - i.unwrap_or(0) > 0 {
                                if i.is_some() {
                                    ui_state.side_menu = SideMenu::Libary(Some(i.unwrap() + 1))
                                } else {
                                    ui_state.side_menu = SideMenu::Libary(Some(0))
                                }
                            }
                        }
                        SideMenu::Queue(_) => {}
                        SideMenu::None => {}
                    },
                    Focus::None => {}
                },
                KeyCode::Up => match ui_state.focus {
                    Focus::MainWindow => match ui_state.main_window_state.clone() {
                        MainWindowState::Results(_) => match ui_state.focused_result {
                            FocusedResult::Song(i) => {
                                if i > 0 {
                                    ui_state.focused_result = FocusedResult::Song(i - 1)
                                }
                            }
                            FocusedResult::Record(i) => {
                                if i > 0 {
                                    ui_state.focused_result = FocusedResult::Record(i - 1)
                                }
                            }
                            FocusedResult::Artist(i) => {
                                if i > 0 {
                                    ui_state.focused_result = FocusedResult::Artist(i - 1)
                                }
                            }
                            _ => {}
                        },
                        MainWindowState::ArtistFocus(a, i) => {
                            if i.is_some() {
                                if i.unwrap() > 0 {
                                    ui_state.main_window_state =
                                        MainWindowState::ArtistFocus(a, Some(i.unwrap() - 1))
                                } else {
                                    ui_state.main_window_state =
                                        MainWindowState::ArtistFocus(a, None)
                                }
                            }
                        }
                        MainWindowState::RecordFocus(r, i) => {
                            if i.is_some() {
                                if i.unwrap() > 0 {
                                    ui_state.main_window_state =
                                        MainWindowState::RecordFocus(r, Some(i.unwrap() - 1))
                                } else {
                                    ui_state.main_window_state =
                                        MainWindowState::RecordFocus(r, None)
                                }
                            }
                        }
                        _ => {}
                    },
                    Focus::SideWindow => match ui_state.side_menu {
                        SideMenu::Libary(i) => {
                            if i.is_some() {
                                if i.unwrap() > 0 {
                                    ui_state.side_menu = SideMenu::Libary(Some(i.unwrap() - 1))
                                } else {
                                    ui_state.side_menu = SideMenu::Libary(None)
                                }
                            }
                        }
                        SideMenu::Queue(_) => {}
                        SideMenu::None => {}
                    },
                    Focus::None => {}
                },
                KeyCode::Char('b') => if matches!(ui_state.focus, Focus::MainWindow) && ui_state.history.len() > 0 {
                    ui_state.main_window_state = ui_state.history.pop_front().unwrap();
                },
                KeyCode::Enter => {
                    match ui_state.focus {
                        Focus::MainWindow => {
                            match ui_state.main_window_state.clone() { 
                                MainWindowState::ArtistFocus(a, i) => if i.is_some() {
                                    ui_state.history.push_front(ui_state.main_window_state.to_owned());
                                    ui_state.main_window_state = MainWindowState::RecordFocus(a.get_albums().get(i.unwrap()).unwrap().to_owned(), None)
                                },
                                MainWindowState::RecordFocus(r, i) => if i.is_some() {
                                    ui_state.history.push_front(ui_state.main_window_state.to_owned());
                                    ui_state.main_window_state = MainWindowState::SongFocus(r.get_songs().get(i.unwrap()).unwrap().to_owned())
                                },
                                _ => {},
                            }
                        },
                        Focus::SideWindow => {
                            match ui_state.side_menu {
                                SideMenu::Libary(i) => if i.is_some() {
                                    ui_state.history.push_front(ui_state.main_window_state.to_owned());
                                    ui_state.focus = Focus::MainWindow;
                                    ui_state.side_menu = SideMenu::Libary(None);
                                    ui_state.main_window_state = MainWindowState::ArtistFocus(ui_state.artists.get(i.unwrap()).unwrap().to_owned(), None);
                                },
                                SideMenu::Queue(_) => {},
                                SideMenu::None => {},
                            }
                        },
                        Focus::None => {}
                    }
                },
                _ => {}
            }
        } else {
            match input.code {
                KeyCode::Char('y') => {
                    match ui_state.main_window_state.clone() {
                        MainWindowState::SongFocus(s) => {
                            let new_s = s.to_owned();
                            thread::spawn(move || {
                                new_s.delete();
                            });
                            music_repository.remove_song(s);
                        }
                        MainWindowState::RecordFocus(r, _) => {
                            let new_r = r.to_owned();
                            thread::spawn(move || {
                                new_r.delete();
                            });
                            music_repository.remove_album(r);
                        }
                        MainWindowState::ArtistFocus(a, _) => {
                            let new_a = a.to_owned();
                            thread::spawn(move || {
                                new_a.delete();
                            });
                            music_repository.remove_artist(a);
                        }
                        _ => {}
                    }
                    ui_state.main_window_state = MainWindowState::Help;
                    ui_state.side_menu = SideMenu::Libary(None);
                    ui_state.delete = false;
                }
                KeyCode::Char('n') => ui_state.delete = false,
                _ => {}
            }
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
        KeyCode::Enter => {
            ui_state.focus = Focus::MainWindow;
            ui_state.main_window_state = MainWindowState::Results((vec![], vec![], vec![]));
        },
        _ => {}
    }
}
