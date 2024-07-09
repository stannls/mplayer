use std::collections::VecDeque;

use super::scroll_components::ScrollTable;
use crate::api::{player::SongInfo, Album, Artist, Song};

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    text::Line,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};

pub enum ToolbarType {
    Default,
    Download,
    Play,
}

// The main window
pub fn build_window_border() -> Block<'static> {
    Block::default().title("mplayer").borders(Borders::ALL)
}

// The menu on the left side
pub fn build_libary(
    content: Vec<String>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let rows = content.into_iter().map(|f| vec![f]).collect();
    ScrollTable::new(rows, vec![Constraint::Percentage(100)])
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title("[L]ibary"))
}

// The welcome window
pub fn build_help_window() -> Table<'static> {
    Table::new(vec![
        Row::new(vec![
            Cell::from("Global").style(Style::default().add_modifier(Modifier::BOLD))
        ]),
        Row::new(vec!["L", "Libary"]),
        Row::new(vec!["Q", "Queue"]),
        Row::new(vec!["s", "Search"]),
        Row::new(vec!["h", "Help"]),
        Row::new(vec!["↑", "Up"]),
        Row::new(vec!["↓", "Down"]),
        Row::new(vec!["<enter>", "Select"]),
        Row::new(vec!["<space>", "Pause/Continue"]),
        Row::new(vec!["v", "Stop"]),
        Row::new(vec!["n", "Skip"]),
        Row::new(vec!["c", "Current album"]),
        Row::new(vec!["b", "Back"]),
        Row::new(vec![
            Cell::from("Search").style(Style::default().add_modifier(Modifier::BOLD))
        ]),
        Row::new(vec!["S", "Select song"]),
        Row::new(vec!["R", "Select record"]),
        Row::new(vec!["A", "Select artist"]),
        Row::new(vec!["P", "Select playlist"]),
        Row::new(vec![
            Cell::from("Media-View").style(Style::default().add_modifier(Modifier::BOLD))
        ]),
        Row::new(vec!["d", "Download media"]),
        Row::new(vec!["p", "Play media"]),
        Row::new(vec!["e", "Enqueue media"]),
    ], &[Constraint::Percentage(20), Constraint::Percentage(80)])
    .block(Block::default().borders(Borders::ALL).title("Help"))
}

// The searchbar
pub fn build_searchbar(searchbar_content: Option<String>) -> Paragraph<'static> {
    let content = match searchbar_content {
        Some(t) => t,
        _ => "Enter search term".to_string(),
    };
    Paragraph::new(content).block(Block::default().borders(Borders::all()).title("[S]earch"))
}

// The Toolbar that is show when selecting a search result
pub fn build_focus_toolbox(toolbar_type: ToolbarType) -> Paragraph<'static> {
    Paragraph::new(match toolbar_type {
        ToolbarType::Download => "[b]ack [d]ownload [↑]up [↓]down [enter]select".to_string(),
        ToolbarType::Play => {
            "[b]ack [p]lay [e]nqueue [↑]up [↓]down [enter]select [D]elete".to_string()
        }
        ToolbarType::Default => "[b]ack [↑]up [↓]down [enter]select [D]elete".to_string(),
    })
}

pub trait SearchEntity {
    fn display(&self) -> String;
}

pub struct EmtpyEntity{}
impl SearchEntity for EmtpyEntity {
    fn display(&self) -> String {
        "".to_string()
    }
}

// Builds a result box based of the SearchEntity Trait
pub fn build_result_box<T: SearchEntity>(
    title: String,
    content: Vec<T>,
    focused_result: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let items = content.into_iter().map(|f| vec![f.display()]).collect();
    ScrollTable::new(items, vec![Constraint::Percentage(100)])
        .focus(focused_result)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title(title))
}

pub fn build_artist_focus(
    artist: Box<dyn Artist>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let mut rows = vec![];
    for r in artist.get_albums() {
        rows.push(vec![r.get_name(), r.get_release_date()]);
    }

    ScrollTable::new(rows, vec![Constraint::Max(u16::MAX), Constraint::Length(20)])
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .block(
            Block::default()
                .borders(Borders::all())
                .title(artist.get_name()),
        )
        .header(Row::new(vec!["Title", "Release Date"]))
}

pub fn build_song_focus(song: Box<dyn Song>) -> Table<'static> {
    let title = format!(
        "{}{}",
        song.get_title(),
        match song.get_disambiguation() {
            Some(str) =>
                if !str.is_empty() {
                    format!(" ({})", str)
                } else {
                    String::new()
                },
            _ => String::new(),
        }
    );
    let content = Row::new(vec![
        String::from("1"),
        title,
        song.get_length().unwrap_or("00:00".to_string()),
    ]);
    return Table::new(vec![content], &[
            Constraint::Length(3),
            Constraint::Max(u16::MAX),
            Constraint::Length(8),
        ])
        .block(
            Block::default()
                .borders(Borders::all())
                .title(song.get_title()),
        )
        .header(Row::new(vec!["#", "Title", "Length"]))
}

pub fn build_record_focus(
    record: Box<dyn Album>,
    index: Option<usize>,
    displayable_results: usize,
    current_song: &Option<SongInfo>,
) -> Table<'static> {
    let playing = if current_song.is_some() {
        record
            .get_songs()
            .iter()
            .position(|x| x.get_title() == current_song.to_owned().unwrap().name)
    } else {
        None
    };
    let rows: Vec<Vec<String>> = record
        .get_songs()
        .into_iter()
        .map(|f| {
            vec![
                f.get_number().unwrap_or("".to_string()),
                f.get_title(),
                f.get_length().unwrap_or("00:00".to_string()),
            ]
        })
        .collect();

    ScrollTable::new(rows, vec![Constraint::Length(3), Constraint::Max(u16::MAX), Constraint::Length(8)])
        .focus(index)
        .selected(playing)
        .displayable_results(displayable_results)
        .render()
        .header(Row::new(vec!["#", "Title", "Length"]))
        .block(
            Block::default()
                .borders(Borders::all())
                .title(record.get_name()),
        )
}

pub fn build_song_info(song_info: &SongInfo) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(format!("{} - {}", song_info.name, song_info.artist)),
        Line::from(vec![
            Span::styled("on ", Style::default().add_modifier(Modifier::ITALIC)),
            Span::raw(song_info.album.to_owned()),
        ]),
        Line::from(format!(
            "{}/{}",
            song_info.played_time().unwrap(),
            song_info.length
        )),
    ])
}

pub fn build_progress_bar(song_info: &SongInfo) -> Gauge<'static> {
    let progress = song_info.played_time().unwrap() as f64 / song_info.length as f64;
    Gauge::default()
        .ratio(if progress < 1.0 { progress } else { 1.0 })
        .gauge_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
        .label(format!(
            "{:0>2}:{:0>2}/{:0>2}:{:0>2}",
            (song_info.played_time().unwrap() / 60) % 60,
            song_info.played_time().unwrap() % 60,
            (song_info.length / 60) % 60,
            song_info.length % 60
        ))
}

pub fn build_queue(
    q: VecDeque<SongInfo>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    ScrollTable::new(q.into_iter().map(|f| vec![f.name]).collect(), vec![Constraint::Percentage(100)])
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title("[Q]ueue"))
}
