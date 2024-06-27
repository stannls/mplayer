use crate::ui::interface::{MainWindowState, UiState};
use super::interface::FocusedResult;
use crate::ui::interface::{MainWindowState, UiState};
use tui::layout::{Constraint, Direction, Layout, Rect};

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
