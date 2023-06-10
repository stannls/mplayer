use crate::api::{Album, Song};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::io::Error;
use std::{io::ErrorKind, sync::Arc};

use super::{AlbumProvider, Downloader, SongProvider};

pub struct BandcampDownloader {}

impl BandcampDownloader {
    pub fn new() -> Arc<BandcampDownloader> {
        Arc::new(BandcampDownloader {})
    }
    fn get_song_page_link(
        title: &str,
        artist: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::new();
        let res: Vec<Result> = client.post("https://bandcamp.com/api/bcsearch_public_api/1/autocomplete_elastic")
            .body(format!("{{\"search_text\":\"{} - {}\",\"search_filter\":\"\",\"full_page\":false,\"fan_id\":null}}", artist, title))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0")
            .header("Referer", "https://bandcamp.com/")
            .header("Content-Type", "application/json")
            .send()?
            .json::<SearchResult>()?
            .auto.results
            .into_iter()
            .filter(|f| f.type_field == "t")
            .filter(|f| f.band_name == artist)
            .collect();
        Ok(res
            .get(0)
            .ok_or(Error::new(std::io::ErrorKind::Other, "No link found"))?
            .item_url_path
            .to_owned())
    }
    fn get_album_page_link(
        album_name: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::new();
        let res: Vec<Result> = client.post("https://bandcamp.com/api/bcsearch_public_api/1/autocomplete_elastic")
            .body(format!("{{\"search_text\":\"{}\",\"search_filter\":\"\",\"full_page\":false,\"fan_id\":null}}", album_name))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0")
            .header("Referer", "https://bandcamp.com/")
            .header("Content-Type", "application/json")
            .send()?
            .json::<SearchResult>()?
            .auto.results
            .into_iter()
            .filter(|f| f.type_field == "a")
            .filter(|f| f.name == album_name)
            .collect();
        Ok(res
            .get(0)
            .ok_or(Error::new(std::io::ErrorKind::Other, "No link found"))?
            .item_url_path
            .to_owned())
    }
    fn extract_audio_links(
        page_link: &str,
    ) -> std::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let html_page = reqwest::blocking::get(page_link)?.text()?;
        let dom = Html::parse_document(&html_page);
        let selector = Selector::parse("script[data-tralbum]").unwrap();
        let element = dom
            .select(&selector)
            .next()
            .ok_or(Error::new(std::io::ErrorKind::Other, "Parsing failed"))?;
        let mut links = vec![];
        for (name, value) in &element.value().attrs {
            if &name.local == "data-tralbum" {
                let data = gjson::get(value.trim(), "@this");
                let link = data
                    .get("trackinfo")
                    .array()
                    .get(0)
                    .ok_or(Error::new(std::io::ErrorKind::Other, "Parsing failed"))?
                    .get("file.mp3-128")
                    .to_string();
                links.push(link);
            }
        }
        Ok(links)
    }
}

impl SongProvider for BandcampDownloader {
    fn provide_song(
        &self,
        recording: Box<dyn Song>,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let artist_name = recording.get_artist_name();
        let track_name = recording.get_title();
        let download_page = BandcampDownloader::get_song_page_link(&track_name, &artist_name)?;
        let results = BandcampDownloader::extract_audio_links(&download_page)?;
        Ok(results
            .get(0)
            .ok_or(Error::new(ErrorKind::Other, "No link found"))?
            .to_owned())
    }
}

impl AlbumProvider for BandcampDownloader {
    fn provide_album(
        &self,
        album: Box<dyn Album + Send + Sync>,
    ) -> std::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let album_page = BandcampDownloader::get_album_page_link(&album.get_name())?;
        BandcampDownloader::extract_audio_links(&album_page)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    pub auto: Auto,
    pub tag: Tag,
    pub genre: Genre,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Auto {
    pub results: Vec<Result>,
    #[serde(rename = "stat_params_for_tag")]
    pub stat_params_for_tag: String,
    #[serde(rename = "time_ms")]
    pub time_ms: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Result {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: i64,
    #[serde(rename = "art_id")]
    pub art_id: i64,
    #[serde(rename = "img_id")]
    pub img_id: Option<String>,
    pub name: String,
    #[serde(rename = "band_id")]
    pub band_id: i64,
    #[serde(rename = "band_name")]
    pub band_name: String,
    #[serde(rename = "album_name")]
    pub album_name: String,
    #[serde(rename = "item_url_root")]
    pub item_url_root: String,
    #[serde(rename = "item_url_path")]
    pub item_url_path: String,
    pub img: String,
    #[serde(rename = "album_id")]
    pub album_id: i64,
    #[serde(rename = "stat_params")]
    pub stat_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tag {
    pub matches: Vec<Option<String>>,
    pub count: i64,
    #[serde(rename = "time_ms")]
    pub time_ms: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Genre {}

impl Downloader for BandcampDownloader {}
