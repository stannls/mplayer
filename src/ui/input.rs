use crossterm::event::{self, KeyCode, KeyEvent};

use super::interface::{Focus, FocusedResult, MainWindowState, SideMenu, UiState};
use crate::ui::helpers;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use std::thread;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct InputHandler {
    handlers: Vec<ConditionalHandler>
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler { handlers: vec![] }
    }
    pub fn register_handler(mut self, handler: ConditionalHandler) -> InputHandler {
        self.handlers.push(handler);
        self
    }
    pub fn handle(&self, input: KeyEvent, ui_state: &mut UiState) {
        self.handlers.iter().for_each(|handler| handler.handle(input, ui_state))
    }
    pub fn load_input_handlers(mut self) -> InputHandler {
        let main_input_handler = ConditionalHandler::new(|ui_state| !ui_state.searching && !ui_state.delete)
            .register_handler(KeyCode::Char('p'), |ui_state| match ui_state.main_window_state.to_owned() {
                MainWindowState::SongFocus(song) => ui_state.music_player.play_song(song, true),
                MainWindowState::RecordFocus(record, _) => ui_state.music_player.play_album(record, true),
                _ => {}
            }
        ).unwrap()
        .register_handler(KeyCode::Char('e'), |ui_state| match ui_state.main_window_state.to_owned() {
            MainWindowState::SongFocus(song) => ui_state.music_player.play_song(song, false),
            MainWindowState::RecordFocus(record, _) => ui_state.music_player.play_album(record, false),
            _ => {}
        }
    ).unwrap()
        .register_handler(KeyCode::Char('D'), |ui_state| match ui_state.main_window_state.to_owned() {
           MainWindowState::ArtistFocus(_, _) | MainWindowState::RecordFocus(_, _) | MainWindowState::SongFocus(_) => ui_state.delete = true,
           _ => {}
        }).unwrap()
        .register_handler(KeyCode::Char(' '), |ui_state| ui_state.music_player.pause())
        .unwrap()
        .register_handler(KeyCode::Char('n'), |ui_state| ui_state.music_player.skip())
        .unwrap()
        .register_handler(KeyCode::Char('v'), |ui_state| ui_state.music_player.stop())
        .unwrap()
        .register_handler(KeyCode::Char('q'), |ui_state| ui_state.quit = true)
        .unwrap()
        .register_handler(KeyCode::Char('+'), |ui_state| ui_state.music_player.change_volume(0.1))
        .unwrap()
        .register_handler(KeyCode::Char('-'), |ui_state| ui_state.music_player.change_volume(-0.1))
        .unwrap()
        .register_handler(KeyCode::Char('h'), |ui_state| {
            ui_state.main_window_state = MainWindowState::Help;
            ui_state.focus = Focus::None;
        }).unwrap()
        .register_handler(KeyCode::Char('c'), |ui_state| if ui_state.music_player.get_song_info().is_some() {
           let current_album = ui_state.music_repository.find_current_album(&ui_state.music_player.get_song_info().unwrap()); 
           if current_album.is_some() { ui_state.main_window_state = MainWindowState::RecordFocus(current_album.unwrap(), None) }
        }).unwrap()
        .register_handler(KeyCode::Char('s'), |ui_state| {
            ui_state.searching = true;
            ui_state.focused_result = FocusedResult::None;
        }).unwrap()
        .register_handler(KeyCode::Char('A'), |ui_state| if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Artist(_)) {
            ui_state.focused_result = FocusedResult::Artist(0);
        }).unwrap()
        .register_handler(KeyCode::Char('S'), |ui_state| if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Song(_)) {
            ui_state.focused_result = FocusedResult::Song(0);
        }).unwrap()
        .register_handler(KeyCode::Char('R'), |ui_state| if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Record(_)) {
            ui_state.focused_result = FocusedResult::Record(0);
        }).unwrap()
        .register_handler(KeyCode::Char('P'), |ui_state| if matches!(ui_state.main_window_state, MainWindowState::Results(_)) && !matches!(ui_state.focused_result, FocusedResult::Playlist(_)) {
            ui_state.focused_result = FocusedResult::Playlist(0);
        }).unwrap()
        .register_handler(KeyCode::Char('L'), |ui_state| if !matches!(ui_state.side_menu, SideMenu::Libary(_)) {
            ui_state.side_menu = SideMenu::Libary(None);
            ui_state.focus = Focus::SideWindow;
            match ui_state.main_window_state.to_owned() {
                MainWindowState::RecordFocus(r, _) => ui_state.main_window_state = MainWindowState::RecordFocus(r, None),
                MainWindowState::ArtistFocus(a, _) => ui_state.main_window_state = MainWindowState::ArtistFocus(a, None),
                _ => {}
            }
        } else {
            ui_state.focus = Focus::SideWindow;
        }).unwrap()
        .register_handler(KeyCode::Char('Q'), |ui_state| if !matches!(ui_state.side_menu, SideMenu::Queue(_)) {
            ui_state.side_menu = SideMenu::Queue(None);
            ui_state.focus = Focus::SideWindow;
            match ui_state.main_window_state.to_owned() {
                MainWindowState::RecordFocus(r, _) => ui_state.main_window_state = MainWindowState::RecordFocus(r, None),
                MainWindowState::ArtistFocus(a, _) => ui_state.main_window_state = MainWindowState::ArtistFocus(a, None),
                _ => {}
            }
        } else {
            ui_state.focus = Focus::SideWindow;
        }).unwrap()
        .register_handler(KeyCode::Down, |ui_state| ui_state.scroll_down()).unwrap()
        .register_handler(KeyCode::Up, |ui_state| ui_state.scroll_up()).unwrap()
        .register_handler(KeyCode::Char('b'), |ui_state| if matches!(ui_state.focus, Focus::MainWindow) && ui_state.history.len() > 0 {
            ui_state.main_window_state = ui_state.history.pop_front().unwrap();
        }).unwrap() 
        .register_handler(KeyCode::Enter, |ui_state| ui_state.enter()).unwrap();
        let search_handler = ConditionalHandler::new(|ui_state| ui_state.searching && !ui_state.delete);
        let delete_handler = ConditionalHandler::new(|ui_state| ui_state.delete);
        self.handlers = vec![main_input_handler, search_handler, delete_handler];
        self
    }
}

pub struct ConditionalHandler {
    handlers: HashMap<KeyCode, Box<dyn Fn(&mut UiState)>>,
    condition: Box<dyn Fn(&UiState) -> bool>
}

impl ConditionalHandler {
    pub fn new<F>(condition: F) -> ConditionalHandler
    where
        F: Fn(&UiState) -> bool +'static
    {
        ConditionalHandler { handlers: HashMap::new(), condition: Box::new(condition) }
    }

    pub fn register_handler<F>(mut self, keycode: KeyCode, handler: F) -> Option<ConditionalHandler> 
    where
        F: Fn(&mut UiState) + 'static
    {
        if self.handlers.contains_key(&keycode) {
            None
        } else {
            self.handlers.insert(keycode, Box::new(handler));
            Some(self)
        }
    }

    pub fn handle(&self, input: KeyEvent, ui_state: &mut UiState) {
        if (*self.condition)(&ui_state.to_owned()) && self.handlers.contains_key(&input.code) {
           self.handlers[&input.code](ui_state) 
        }
    }
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

pub(crate) async fn handle_input(input: KeyEvent, ui_state: &mut UiState) {
    // Match arm for inputting text
    if ui_state.searching {
        handle_search_input(input, ui_state).await
    } else {
        if !ui_state.delete {
            // Match arm for everything else
            match input.code {
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
                            ui_state.music_repository.remove_song(s);
                        }
                        MainWindowState::RecordFocus(r, _) => {
                            let new_r = r.to_owned();
                            thread::spawn(move || {
                                new_r.delete();
                            });
                            ui_state.music_repository.remove_album(r);
                        }
                        MainWindowState::ArtistFocus(a, _) => {
                            let new_a = a.to_owned();
                            thread::spawn(move || {
                                new_a.delete();
                            });
                            ui_state.music_repository.remove_artist(a);
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
