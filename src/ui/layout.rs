use tui::layout::{Constraint, Direction, Layout, Rect};

pub fn build_main_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
}

pub fn build_content_layout() -> Layout {
    Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Length(30), Constraint::Min(2)].as_ref())
}

pub fn build_search_layout(parent_layout: Rect) -> Vec<Rect> {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(parent_layout);
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
        .collect()
}
