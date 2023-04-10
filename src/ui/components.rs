use super::scroll_components::ScrollTable;
use crate::api::search::wrapper;
use crate::ui::helpers::select_correct_media;
use chrono::Duration;
use musicbrainz_rs::entity::{artist, release, release_group::ReleaseGroup};
use tui::{
    layout::Constraint,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

// The main window
pub fn build_window_border() -> Block<'static> {
    Block::default().title("mplayer").borders(Borders::ALL)
}

// The menu on the left side
pub fn build_side_menu() -> Paragraph<'static> {
    Paragraph::new("Albums").block(Block::default().borders(Borders::ALL).title("Libary"))
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
        _ => "Enter search term [s]".to_string(),
    };
    Paragraph::new(content).block(Block::default().borders(Borders::all()).title("Search"))
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

pub fn build_song_focus(song: wrapper::Recording) -> Table<'static> {
    let t = Duration::milliseconds(song.data.length.unwrap() as i64);
    let title = format!(
        "{}{}",
        song.data.title.clone(),
        match song.data.disambiguation {
            Some(str) =>
                if str != "" {
                    format!(" ({})", str)
                } else {
                    format!("")
                },
            _ => format!(""),
        }
    );
    let content = Row::new(vec![
        String::from("1"),
        title,
        format!("{}:{}", (t.num_seconds() / 60) % 60, t.num_seconds() % 60),
    ]);
    return Table::new(vec![content])
        .block(
            Block::default()
                .borders(Borders::all())
                .title(song.data.title),
        )
        .header(Row::new(vec!["#", "Title", "Length"]))
        .widths(&[
            Constraint::Percentage(2),
            Constraint::Percentage(90),
            Constraint::Percentage(8),
        ]);
}

pub fn build_record_focus(
    record: release::Release,
    index: Option<usize>,
    displayable_results: usize,
) -> Table<'static> {
    let rows: Vec<Vec<String>> = select_correct_media(record.media.to_owned().unwrap())
        .tracks
        .to_owned()
        .unwrap()
        .into_iter()
        .map(|f| {
            let t = Duration::milliseconds(f.length.unwrap() as i64);
            vec![
                f.number,
                f.title,
                format!("{}:{}", (t.num_seconds() / 60) % 60, t.num_seconds() % 60),
            ]
        })
        .collect();

    ScrollTable::new(rows)
        .focus(index)
        .displayable_results(displayable_results)
        .render()
        .header(Row::new(vec!["#", "Title", "Length"]))
        .block(Block::default().borders(Borders::all()).title(record.title))
        .widths(&[
            Constraint::Percentage(2),
            Constraint::Percentage(90),
            Constraint::Percentage(8),
        ])
}
