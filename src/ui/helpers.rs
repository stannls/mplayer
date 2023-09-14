use crate::api::search::remote;
use crate::ui::interface::{MainWindowState, UiState};
use tokio::task;
use super::interface::FocusedResult;
use tui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) async fn query_web(ui_state: &mut UiState) {
    // Query up results first
    //
    // The cloning of the data is really ugly but I found no other way because of the async tasks
    let artists = task::spawn(remote::search_artists(
            ui_state.searchbar_content.to_owned(),
            ));
    let albums = task::spawn(remote::search_albums(ui_state.searchbar_content.to_owned()));
    let titles = task::spawn(remote::search_songs(ui_state.searchbar_content.to_owned()));

    ui_state.searching = false;
    ui_state.searchbar_content = String::from("");
    ui_state.main_window_state = MainWindowState::Results((
            albums.await.unwrap().unwrap(),
            artists.await.unwrap().unwrap(),
            titles.await.unwrap().unwrap(),
            ));
}

pub(crate) fn check_scroll_space_down(ui_state: &UiState) -> bool {
    match ui_state.main_window_state.clone() {
        MainWindowState::Results(elements) => match ui_state.focused_result {
            FocusedResult::Song(id) => elements.2.len() - 1 > id,
            FocusedResult::Artist(id) => elements.1.len() - 1 > id,
            FocusedResult::Record(id) => elements.0.len() - 1 > id,
            _ => false,
        },
        _ => false,
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
            )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
            )
        .split(popup_layout[1])[1]
}
