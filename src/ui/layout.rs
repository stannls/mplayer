use tui::layout::{Layout, Constraint, Direction};

pub fn build_main_layout() -> Layout{
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1) 
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
}

pub fn build_content_layout() -> Layout{
    Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Length(30), Constraint::Min(2)].as_ref())
}
