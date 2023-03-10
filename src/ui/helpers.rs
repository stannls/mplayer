use tokio::task;
use crate::api::search::remote; 
use crate::ui::interface::{UiState, MainWindowState};

pub(crate) async fn query_web(ui_state: &mut UiState){
    // Query up results first
    //
    // The cloning of the data is really ugly but I found no other way because of the async tasks
    let artists = task::spawn(remote::search_artists(ui_state.searchbar_content.to_owned()));
    let albums = task::spawn(remote::search_albums(ui_state.searchbar_content.to_owned()));
    let titles = task::spawn(remote::search_titles(ui_state.searchbar_content.to_owned()));

    ui_state.searching = false;
    ui_state.searchbar_content = String::from("");
    ui_state.main_window_state = MainWindowState::Results((albums.await.unwrap().unwrap(), artists.await.unwrap().unwrap(), titles.await.unwrap().unwrap()));
}
