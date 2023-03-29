use crate::api::search::wrapper;
use crate::ui::helpers::select_correct_media;
use chrono::Duration;
use musicbrainz_rs::entity::{artist, release, release_group::ReleaseGroup};
use tui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

pub fn build_window_border() -> Block<'static> {
    Block::default().title("mplayer").borders(Borders::ALL)
}

pub fn build_side_menu() -> Paragraph<'static> {
    Paragraph::new("Albums").block(Block::default().borders(Borders::ALL).title("Libary"))
}

pub fn build_main_window() -> Paragraph<'static> {
    Paragraph::new("Lorem ipsum dolor sit amet.")
        .block(Block::default().borders(Borders::ALL).title("Welcome"))
}

pub fn build_searchbar(searching: &bool, searchbar_content: &String) -> Paragraph<'static> {
    Paragraph::new(if *searching {
        searchbar_content.clone()
    } else {
        String::from("Enter search term [s]")
    })
    .block(Block::default().borders(Borders::all()).title("Search"))
}

pub fn build_focus_toolbox(download: bool) -> Paragraph<'static> {
    Paragraph::new(if download {
        String::from("[b]ack [d]ownload [↑]up [↓]down [enter]select")
    } else {
        String::from("[b]ack [↑]up [↓]down [enter]select")
    })
}

pub fn build_result_box<T: wrapper::SearchEntity>(
    title: String,
    content: Vec<T>,
    focused_result: Option<usize>,
    displayable_results: usize,
) -> List<'static> {
    match focused_result {
        None => {
            let items: Vec<ListItem> = content
                .into_iter()
                .map(|f| ListItem::new(f.display()))
                .collect();
            List::new(items).block(Block::default().borders(Borders::all()).title(title))
        }
        Some(id) => {
            if content.len() < 1 {
                return List::new(vec![])
                    .block(Block::default().borders(Borders::all()).title(title));
            }
            let mut items: Vec<ListItem> = content[0..id]
                .into_iter()
                .map(|f| ListItem::new(f.display()))
                .collect();
            items.push(
                ListItem::new(content[id].display())
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            );
            items.append(&mut Vec::from_iter(
                content[id + 1..]
                    .into_iter()
                    .map(|f| ListItem::new(f.display())),
            ));
            if id > displayable_results {
                items.drain(0..id - displayable_results);
            }
            List::new(items).block(Block::default().borders(Borders::all()).title(title))
        }
    }
}

pub fn build_artist_focus(
    artist: artist::Artist,
    release_groups: Vec<ReleaseGroup>,
    index: Option<usize>,
) -> Table<'static> {
    let mut rows = vec![];
    for r in release_groups {
        rows.push(Row::new(vec![
            r.title.to_owned(),
            r.first_release_date.unwrap().to_string(),
        ]));
    }
    match index {
        Some(i) => {
            let highlight = rows
                .get(i)
                .unwrap()
                .to_owned()
                .style(Style::default().add_modifier(Modifier::BOLD));
            rows[i] = highlight.clone();
        }
        _ => {}
    }

    return Table::new(rows)
        .block(Block::default().borders(Borders::all()).title(artist.name))
        .header(Row::new(vec!["Title", "Release Date"]))
        .widths(&[Constraint::Percentage(80), Constraint::Percentage(20)]);
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

pub fn build_record_focus(record: release::Release, index: Option<usize>) -> Table<'static> {
    let mut rows: Vec<Row> = select_correct_media(record.media.to_owned().unwrap())
        .tracks
        .to_owned()
        .unwrap()
        .into_iter()
        .map(|f| {
            let t = Duration::milliseconds(f.length.unwrap() as i64);
            Row::new(vec![
                f.number,
                f.title,
                format!("{}:{}", (t.num_seconds() / 60) % 60, t.num_seconds() % 60),
            ])
        })
        .collect();
    match index {
        Some(i) => {
            let highlight = rows
                .get(i)
                .unwrap()
                .to_owned()
                .style(Style::default().add_modifier(Modifier::BOLD));
            rows[i] = highlight.clone();
        }
        _ => {}
    }
    return Table::new(rows)
        .header(Row::new(vec!["#", "Title", "Length"]))
        .block(Block::default().borders(Borders::all()).title(record.title))
        .widths(&[
            Constraint::Percentage(2),
            Constraint::Percentage(90),
            Constraint::Percentage(8),
        ]);
}
