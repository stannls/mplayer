use super::components::ToolbarType;
use super::helpers;
use super::input::handle_input;
use super::input::ConditionalHandler;
use super::input::Event;
use super::input::InputHandler;
use crate::api::fs::MusicRepository;
use crate::api::player::MusicPlayer;
use crate::api::Artist;
use crate::api::{Album, Song};
use crate::ui::components::EmtpyEntity;
use crate::ui::{components, layout};
use crossterm::event::EnableMouseCapture;
use crossterm::event::{DisableMouseCapture, KeyEvent};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use itertools::Itertools;
use std::collections::VecDeque;
use std::io;
use std::{io::Stdout, sync::mpsc::Receiver};
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Clear;
use tui::widgets::Paragraph;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Clone)]
pub(crate) struct UiState {
    pub(crate) searching: bool,
    pub(crate) searchbar_content: String,
    pub(crate) quit: bool,
    pub(crate) main_window_state: MainWindowState,
    pub(crate) focused_result: FocusedResult,
    pub(crate) history: VecDeque<MainWindowState>,
    pub(crate) artists: Vec<Box<dyn Artist + Send + Sync>>,
    pub(crate) side_menu: SideMenu,
    pub(crate) focus: Focus,
    pub(crate) delete: bool,
    pub(crate) music_player: MusicPlayer,
    pub(crate) music_repository: MusicRepository,
}

impl UiState {
    pub fn scroll_down(&mut self) {
        match self.focus {
            Focus::MainWindow => match self.main_window_state.to_owned() {
                MainWindowState::Results(_) => match self.focused_result {
                    FocusedResult::Song(i) => if helpers::check_scroll_space_down(&self) {self.focused_result = FocusedResult::Song(i+1)},
                    FocusedResult::Record(i) => if helpers::check_scroll_space_down(&self) {self.focused_result = FocusedResult::Record(i+1)},
                    FocusedResult::Artist(i) => if helpers::check_scroll_space_down(&self) {self.focused_result = FocusedResult::Artist(i+1)},
                    _ => {}
                },
                MainWindowState::ArtistFocus(a, i) => {
                    if i.is_some() && a.get_albums().len() - i.unwrap() > 1 {
                        self.main_window_state = MainWindowState::ArtistFocus(a, i.map(|v| v+1));
                    } else if i.is_none() && a.get_albums().len() > 0 {
                        self.main_window_state = MainWindowState::ArtistFocus(a, Some(0));
                    }
                },
                MainWindowState::RecordFocus(r, i) => {
                    if i.is_some() && r.get_songs().len() - i.unwrap() > 1 {
                        self.main_window_state = MainWindowState::RecordFocus(r, i.map(|v| v+1));
                    } else if i.is_none() && r.get_songs().len() > 0 {
                        self.main_window_state = MainWindowState::RecordFocus(r, Some(0));
                    }
                },
                _ => {}
            },
            Focus::SideWindow => match self.side_menu {
               SideMenu::Libary(i) => if i.is_some() && self.artists.len() - i.unwrap() > 1 {
                   self.side_menu = SideMenu::Libary(i.map(|v| v+1));
               } else {
                    self.side_menu = SideMenu::Libary(Some(0));
               },
               _ => {}
            },
            _ => {}
        }
    }
    pub fn scroll_up(&mut self) {
        match self.focus {
            Focus::MainWindow => match self.main_window_state.to_owned() {
                MainWindowState::Results(_) => match self.focused_result.to_owned() {
                    FocusedResult::Song(i) => if i > 0 {self.focused_result = FocusedResult::Song(i - 1)},
                    FocusedResult::Record(i) => if i > 0 {self.focused_result = FocusedResult::Record(i - 1)},
                    FocusedResult::Artist(i) => if i > 0 {self.focused_result = FocusedResult::Artist(i - 1)},
                    _ => {}
                },
                MainWindowState::ArtistFocus(a, i) => if i.is_some() {
                    if i.unwrap() > 0 {
                        self.main_window_state = MainWindowState::ArtistFocus(a, i.map(|v| v-1));
                    } else {
                        self.main_window_state = MainWindowState::ArtistFocus(a, None);
                    }
                },
                MainWindowState::RecordFocus(r, i) => if i.is_some() {
                    if i.unwrap() > 0 {
                        self.main_window_state = MainWindowState::RecordFocus(r, i.map(|v| v-1));
                    } else {
                        self.main_window_state = MainWindowState::RecordFocus(r, None);
                    }
                },
                _ => {}
            },
            Focus::SideWindow => match self.side_menu {
                SideMenu::Libary(i) => if i.is_some() {
                    if i.unwrap() > 0 {self.side_menu = SideMenu::Libary(i.map(|v| v-1))}
                    else {self.side_menu = SideMenu::Libary(None)}
                },
                _ => {}
            },
            _ => {}
        } 
    }
    pub fn enter(&mut self) {
        match self.focus {
            Focus::MainWindow => match self.main_window_state.to_owned() {
                MainWindowState::ArtistFocus(a, i) => if i.is_some() {
                    self.history.push_front(self.main_window_state.to_owned());
                    self.main_window_state = MainWindowState::RecordFocus(a.get_albums().get(i.unwrap()).unwrap().to_owned(), None);
                },
                MainWindowState::RecordFocus(r, i) => if i.is_some() {
                    self.history.push_front(self.main_window_state.to_owned());
                    self.main_window_state = MainWindowState::SongFocus(r.get_songs().get(i.unwrap()).unwrap().to_owned());
                },
                _ => {}
            },
            Focus::SideWindow => {
                match self.side_menu {
                    SideMenu::Libary(i) => if i.is_some() {
                        self.history.push_front(self.main_window_state.to_owned());
                        self.focus = Focus::MainWindow;
                        self.side_menu = SideMenu::Libary(None);
                        self.main_window_state = MainWindowState::ArtistFocus(self.artists.get(i.unwrap()).unwrap().to_owned(), None);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}

#[derive(Clone)]
pub(crate) enum MainWindowState {
    Help,
    Results((Vec<()>, Vec<()>, Vec<()>)),
    SongFocus(Box<dyn Song + Send + Sync>),
    ArtistFocus(Box<dyn Artist + Send + Sync>, Option<usize>),
    RecordFocus(Box<dyn Album + Send + Sync>, Option<usize>),
}

#[derive(Clone)]
pub(crate) enum FocusedResult {
    None,
    Song(usize),
    Record(usize),
    Artist(usize),
    Playlist(usize),
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) enum SideMenu {
    Libary(Option<usize>),
    Queue(Option<usize>),
    None,
}

#[derive(Clone)]
pub enum Focus {
    MainWindow,
    SideWindow,
    None,
}

impl UiState {
    fn new(music_player: MusicPlayer, music_repository: MusicRepository) -> UiState {
        UiState {
            searching: false,
            searchbar_content: String::from(""),
            quit: false,
            main_window_state: MainWindowState::Help,
            focused_result: FocusedResult::None,
            history: VecDeque::new(),
            artists: vec![],
            side_menu: SideMenu::Libary(None),
            focus: Focus::None,
            delete: false,
            music_player,
            music_repository,
        }
    }
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
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
    terminal.show_cursor()?;
    Ok(())
}

pub async fn render_interface(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    rx: Receiver<Event<KeyEvent>>,
) {
    // Init for ui state and the downloader
    let music_player = MusicPlayer::new();
    let music_dir = dirs::audio_dir().unwrap();
    let mut music_repository = MusicRepository::new(music_dir);
    
    let _ = music_repository.load_cached_artists(); 
    music_repository.watch_files();

    let mut ui_state = UiState::new(music_player, music_repository);

    let handler = InputHandler::new().load_input_handlers();

    // Main UI render loop
    while !ui_state.quit {
        terminal
            .draw(|f| {
                let current_song = ui_state.music_player.get_song_info();

                // Layouting
                let size = f.size();
                let main_layout = layout::build_main_layout(current_song.is_some()).split(size);
                let content_layout = layout::build_content_layout().split(main_layout[1]);
                let focus_layout = layout::build_focus_layout().split(content_layout[1]);
                let result_layout = layout::build_search_layout(content_layout[1]);

                // Main window border
                f.render_widget(components::build_window_border(), size);

                // Searchbar
                let search = if ui_state.searching {
                    Some(ui_state.searchbar_content.to_owned())
                } else {
                    None
                };
                f.render_widget(components::build_searchbar(search), main_layout[0]);

                // Side menu
                match ui_state.side_menu {
                    SideMenu::Libary(i) => f.render_widget(
                        components::build_libary(
                            ui_state
                                .artists
                                .to_owned()
                                .into_iter()
                                .map(|f| f.get_name())
                                .collect_vec(),
                            i,
                            content_layout[0].height as usize - 3,
                        ),
                        content_layout[0],
                    ),
                    SideMenu::Queue(i) => f.render_widget(
                        components::build_queue(
                            ui_state.music_player.get_queue(),
                            i,
                            content_layout[0].height as usize - 3,
                        ),
                        content_layout[0],
                    ),
                    SideMenu::None => {}
                }

                // The main content window
                match ui_state.main_window_state.to_owned() {
                    // The default welcome screen
                    MainWindowState::Help => {
                        f.render_widget(components::build_help_window(), content_layout[1])
                    }
                    // The window for viewing details to a song
                    MainWindowState::SongFocus(s) => {
                        f.render_widget(
                            components::build_song_focus(s.to_owned()),
                            content_layout[1],
                        );
                        if s.is_local() {
                            f.render_widget(
                                components::build_focus_toolbox(ToolbarType::Play),
                                focus_layout[1],
                            );
                        } else {
                            f.render_widget(
                                components::build_focus_toolbox(ToolbarType::Download),
                                focus_layout[1],
                            );
                        }
                    }
                    // The window for viewing details to a record
                    MainWindowState::RecordFocus(r, index) => {
                        f.render_widget(
                            components::build_record_focus(
                                r.to_owned(),
                                index,
                                content_layout[1].height as usize - 3,
                                &current_song,
                            ),
                            content_layout[1],
                        );
                        if r.is_local() {
                            f.render_widget(
                                components::build_focus_toolbox(ToolbarType::Play),
                                focus_layout[1],
                            );
                        } else {
                            f.render_widget(
                                components::build_focus_toolbox(ToolbarType::Download),
                                focus_layout[1],
                            );
                        }
                    }
                    // The window for viewing details to an artist
                    MainWindowState::ArtistFocus(a, index) => {
                        f.render_widget(
                            components::build_artist_focus(
                                a,
                                index,
                                content_layout[1].height as usize - 3,
                            ),
                            content_layout[1],
                        );
                        f.render_widget(
                            components::build_focus_toolbox(ToolbarType::Default),
                            focus_layout[1],
                        );
                    }
                    MainWindowState::Results(_t) => {
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
                        f.render_widget(
                            components::build_result_box(
                                "[S]ong".to_string(),
                                vec![EmtpyEntity {}],
                                scroll_value.0,
                                displayable_results,
                            ),
                            result_layout[0],
                        );
                        // Record search results
                        f.render_widget(
                            components::build_result_box(
                                "[R]ecord".to_string(),
                                vec![EmtpyEntity {}],
                                scroll_value.1,
                                displayable_results,
                            ),
                            result_layout[2],
                        );
                        // Artist search results
                        f.render_widget(
                            components::build_result_box(
                                "[A]rtist".to_string(),
                                vec![EmtpyEntity {}],
                                scroll_value.2,
                                displayable_results,
                            ),
                            result_layout[1],
                        );
                        // Playlsist search results (Not implemented)
                        f.render_widget(
                            components::build_result_box(
                                "[P]laylist".to_string(),
                                vec![EmtpyEntity {}],
                                scroll_value.3,
                                displayable_results,
                            ),
                            result_layout[3],
                        );
                    }
                }

                // The song info of the currently played song
                if current_song.is_some() {
                    let play_layout = layout::build_play_layout().split(main_layout[2]);
                    let current_song = current_song.unwrap();
                    let song_info = components::build_song_info(&current_song);
                    f.render_widget(song_info, play_layout[0]);
                    f.render_widget(
                        components::build_progress_bar(&current_song),
                        play_layout[1],
                    )
                }
                ui_state.artists = ui_state.music_repository.get_artists();
                if ui_state.delete {
                    let text = vec![
                        Spans::from(vec![Span::raw(
                            "Are you sure that you want to delete that?",
                        )]),
                        Spans::from(vec![]),
                        Spans::from(vec![
                            Span::styled("[y]es", Style::default().add_modifier(Modifier::BOLD)),
                            Span::styled(" [n]o", Style::default().add_modifier(Modifier::BOLD)),
                        ]),
                    ];
                    let block = Paragraph::new(text).block(
                        Block::default()
                            .title("Delete Confirmation")
                            .borders(Borders::all()),
                    );
                    let area = helpers::centered_rect(60, 20, size);
                    f.render_widget(Clear, area);
                    f.render_widget(block, area);
                }
            })
            .unwrap();

        // Handles keyboard input
        match rx.recv().unwrap() {
            Event::Input(event) => handler.handle(event, &mut ui_state),
            _ => {}
        }
    }
    let _ = ui_state.music_repository.cache_artists();
}
