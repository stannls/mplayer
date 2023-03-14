
use tui::{widgets::{Block, Borders, Paragraph, List, ListItem}, style::{Style, Modifier}};
use crate::api::search::wrapper;

pub fn build_window_border() -> Block<'static>{
    Block::default().title("mplayer").borders(Borders::ALL)
}

pub fn build_side_menu() -> Paragraph<'static>{
    Paragraph::new("Albums")
        .block(Block::default().borders(Borders::ALL).title("Libary"))
}

pub fn build_main_window() -> Paragraph<'static>{
    Paragraph::new("Lorem ipsum dolor sit amet.")
        .block(Block::default().borders(Borders::ALL).title("Welcome"))

}

pub fn build_searchbar(searching: &bool, searchbar_content: &String) -> Paragraph<'static>{
    Paragraph::new(if *searching {searchbar_content.clone()} else {String::from("Enter search term [s]")})
        .block(Block::default().borders(Borders::all()).title("Search"))
}

pub fn build_result_box<T: wrapper::SearchEntity>(title: String, content: Vec<T>, focused_result: Option<usize>, displayable_results: usize) -> List<'static>{
    match focused_result{
        None => {
            let items: Vec<ListItem> = content.into_iter()
                .map(|f| ListItem::new(f.display()))
                .collect();
            List::new(items)
                .block(Block::default().borders(Borders::all()).title(title))
        },
        Some(id) => {
            if content.len()<1{
                return List::new(vec![])
                .block(Block::default().borders(Borders::all()).title(title));
            }
            let mut items: Vec<ListItem> = content[0..id]
                .into_iter()
                .map(|f| ListItem::new(f.display()))
                .collect();
            items.push(ListItem::new(content[id].display()).style(Style::default().add_modifier(Modifier::BOLD)));
            items.append(&mut Vec::from_iter(content[id+1..]
                        .into_iter()
                        .map(|f| ListItem::new(f.display()))
                ));
            if id>displayable_results {
                items.drain(0..id-displayable_results);
            }
            List::new(items)
                .block(Block::default().borders(Borders::all()).title(title))

        }
    }
    
}
