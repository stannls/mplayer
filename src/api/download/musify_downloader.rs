use crate::api::download::SongProvider;
use crate::api::{Album, Song};
use markup5ever::interface::QualName;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::sync::Arc;
use string_cache::Atom;

use super::{AlbumProvider, Downloader};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResult {
    id: String,
    label: String,
    value: String,
    category: String,
    image: String,
    url: String,
}

enum PageType {
    Song,
    Album,
}

pub struct MusifyDownloader {}

impl MusifyDownloader {
    pub fn new() -> Arc<MusifyDownloader> {
        Arc::new(MusifyDownloader {})
    }

    fn get_page_link(
        song_title: String,
        page_type: PageType,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::new();
        let songs = client
            .get(format!(
                "https://musify.club/search/suggestions?term={}",
                song_title
            ))
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0",
            )
            .header("Referer", "https://musify.club/")
            .send()?
            .json::<Vec<SearchResult>>()?
            .into_iter()
            .filter(|f| {
                f.category
                    == match page_type {
                        PageType::Song => "Треки",
                        PageType::Album => "Релизы",
                    }
            })
            .collect::<Vec<SearchResult>>();
        if songs.len() > 0 {
            Ok(songs.get(0).unwrap().url.to_owned())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No page link found.",
            )))
        }
    }

    pub fn get_links_from_album_page(
        album_url: String,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::new();
        let page = client
            .get(format!("https://musify.club{}", album_url))
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0",
            )
            .header("Referer", "https://musify.club/")
            .send()?
            .text()?;
        let dom = Html::parse_document(&page);

        let playlist_selector = Selector::parse("div.playlist>div").unwrap();
        let playlist_control_selector = Selector::parse("div.playlist__control").unwrap();
        let playlist_items = dom
            .select(&playlist_selector)
            .map(|f| {
                f.select(&playlist_control_selector)
                    .next()
                    .unwrap()
                    .value()
                    .attrs
                    .get(&QualName {
                        prefix: None,
                        ns: Atom::from(""),
                        local: Atom::from("data-url"),
                    })
            })
            .filter(|f| f.is_some())
            .map(|f| f.unwrap().to_string())
            .map(|f| format!("https://musify.club{}", f))
            .collect();
        Ok(playlist_items)
    }
}

impl SongProvider for MusifyDownloader {
    fn provide_song(
        &self,
        recording: Box<dyn Song>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!(
            "{} - {}",
            recording.get_artist_name(),
            recording.get_title()
        );
        Ok(MusifyDownloader::get_page_link(query, PageType::Song)?)
    }
}

impl AlbumProvider for MusifyDownloader {
    fn provide_album(
        &self,
        album: Box<dyn Album + Send + Sync>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let page_link = MusifyDownloader::get_page_link(format!("{}-{}", album.get_artist_name(), album.get_name()), PageType::Album)?;
        MusifyDownloader::get_links_from_album_page(page_link)
    }
}

impl Downloader for MusifyDownloader {}
