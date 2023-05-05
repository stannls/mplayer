use tui::layout::{Constraint, Direction, Layout, Rect};

pub fn build_main_layout(playing: bool) -> Layout {
    if playing {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
    }
}

pub fn build_content_layout() -> Layout {
    Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Length(30), Constraint::Min(1)].as_ref())
}

pub fn build_search_layout(parent_layout: Rect) -> Vec<Rect> {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(parent_layout.to_owned());
    Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(split[0])
        .into_iter()
        .chain(
            Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(split[1])
                .into_iter(),
        )
        .map(|f| *f)
        .collect()
}

pub fn build_focus_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(99), Constraint::Percentage(1)].as_ref())
}

pub fn build_play_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
}
