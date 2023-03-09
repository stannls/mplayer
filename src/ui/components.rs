use tui::widgets::{Block, Borders, Paragraph};

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

pub fn build_searchbar(searching: bool, searchbar_content: &String) -> Paragraph<'static>{
    Paragraph::new(if searching {searchbar_content.clone()} else {String::from("Enter search term [s]")})
        .block(Block::default().borders(Borders::all()).title("Search"))
}
