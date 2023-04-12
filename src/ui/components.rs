use super::scroll_components::ScrollTable;
use crate::api::{search::wrapper, Album, Song};

use musicbrainz_rs::entity::{artist, release_group::ReleaseGroup};
use tui::{
    layout::Constraint,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

// The main window
pub fn build_window_border() -> Block<'static> {
    Block::default().title("mplayer").borders(Borders::ALL)
}

// The menu on the left side
pub fn build_side_menu(
    content: Vec<String>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let rows = content.into_iter().map(|f| vec![f]).collect();
    ScrollTable::new(rows)
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title("[L]ibary"))
        .widths(&[Constraint::Percentage(100)])
}

// The welcome window
pub fn build_welcome_window() -> Paragraph<'static> {
    Paragraph::new("Lorem ipsum dolor sit amet.")
        .block(Block::default().borders(Borders::ALL).title("Welcome"))
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
pub fn build_focus_toolbox(download: bool) -> Paragraph<'static> {
    Paragraph::new(if download {
        "[b]ack [d]ownload [↑]up [↓]down [enter]select".to_string()
    } else {
        "[b]ack [↑]up [↓]down [enter]select".to_string()
    })
}

// Builds a result box based of the SearchEntity Trait
pub fn build_result_box<T: wrapper::SearchEntity>(
    title: String,
    content: Vec<T>,
    focused_result: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let items = content.into_iter().map(|f| vec![f.display()]).collect();
    ScrollTable::new(items)
        .focus(focused_result)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title(title))
        .widths(&[Constraint::Percentage(100)])
}

pub fn build_artist_focus(
    artist: artist::Artist,
    release_groups: Vec<ReleaseGroup>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let mut rows = vec![];
    for r in release_groups {
        rows.push(vec![
            r.title.to_owned(),
            r.first_release_date.unwrap().to_string(),
        ]);
    }

    ScrollTable::new(rows)
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .block(Block::default().borders(Borders::all()).title(artist.name))
        .header(Row::new(vec!["Title", "Release Date"]))
        .widths(&[Constraint::Percentage(80), Constraint::Percentage(20)])
}

pub fn build_song_focus(song: Box<dyn Song>) -> Table<'static> {
    let title = format!(
        "{}{}",
        song.get_title(),
        match song.get_disambiguation() {
            Some(str) =>
                if str != "" {
                    format!(" ({})", str)
                } else {
                    format!("")
                },
            _ => format!(""),
        }
    );
    let content = Row::new(vec![String::from("1"), title, song.get_length()]);
    return Table::new(vec![content])
        .block(
            Block::default()
                .borders(Borders::all())
                .title(song.get_title()),
        )
        .header(Row::new(vec!["#", "Title", "Length"]))
        .widths(&[
            Constraint::Percentage(2),
            Constraint::Percentage(90),
            Constraint::Percentage(8),
        ]);
}

pub fn build_record_focus(
    record: Box<dyn Album>,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let rows: Vec<Vec<String>> = record
        .get_songs()
        .into_iter()
        .map(|f| {
            vec![
                f.get_number().unwrap_or("".to_string()),
                f.get_title(),
                f.get_length(),
            ]
        })
        .collect();

    ScrollTable::new(rows)
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .header(Row::new(vec!["#", "Title", "Length"]))
        .block(
            Block::default()
                .borders(Borders::all())
                .title(record.get_name()),
        )
        .widths(&[
            Constraint::Percentage(2),
            Constraint::Percentage(90),
            Constraint::Percentage(8),
        ])
}
