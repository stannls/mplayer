use crate::api::{Song, Album};
use crate::api::download::SongDownloader;
use std::sync::Arc;
use std::fs::File;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::io::Cursor;
use regex::Regex;
use async_trait::async_trait;
use markup5ever::interface::QualName;
use string_cache::Atom;

use super::{AlbumDownloader, Downloader};



#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResult{
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
    async fn download_from_link(link: String, filename: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>>{
        let response = reqwest::get(&link).await?;
        let path = format!("/tmp/{}.mp3", filename);
        let mut file = File::create(&path)?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(path)
    }

    async fn get_page_link(song_title: String, page_type: PageType) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let songs = client.get(format!("https://musify.club/search/suggestions?term={}", song_title))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0")
            .header("Referer", "https://musify.club/")
            .send()
            .await?
            .json::<Vec<SearchResult>>().await?
            .into_iter()
            .filter(|f| f.category == match page_type {
                PageType::Song => "Треки",
                PageType::Album => "Релизы",
            })
        .collect::<Vec<SearchResult>>();
        if songs.len() > 0 {
            Ok(songs.get(0).unwrap().url.to_owned())
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No page link found.")))
        }
    }

    fn parse_page_link(page_link: String) ->  Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let re = Regex::new(r"^/track/(.*)-(\d*)$")?;
        let captures = re.captures(&page_link).unwrap();
        Ok((format!("https://musify.club/track/dl/{}/{}.mp3", captures.get(2).unwrap().as_str(), captures.get(1).unwrap().as_str()), String::from(captures.get(1).unwrap().as_str())))
    }

    pub async fn get_links_from_album_page(album_url: String) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let page = client.get(format!("https://musify.club{}", album_url))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0")
            .header("Referer", "https://musify.club/")
            .send()
            .await?
            .text()
            .await?;
        let dom = Html::parse_document(&page);
        let playlist_selector = Selector::parse("div.playlist>div").unwrap();
        let playlist_control_selector = Selector::parse("div.playlist__control").unwrap();
        let playlist_items = dom.select(&playlist_selector)
            .map(|f| f.select(&playlist_control_selector).next().unwrap().value().attrs.get(&QualName { prefix: None, ns: Atom::from(""), local: Atom::from("data-url") }).unwrap().to_string())
            .map(|f| format!("https://musify.club{}", f))
            .collect();
        Ok(playlist_items)
    }

}


#[async_trait]
impl SongDownloader for MusifyDownloader {
    async fn download_song(&self, recording: Box<dyn Song>) ->  Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("{} - {}", recording.get_artist_name(), recording.get_title());
        let page_link = MusifyDownloader::get_page_link(query, PageType::Song).await?;
        let (download_link, filename) = MusifyDownloader::parse_page_link(page_link)?;
        Ok(MusifyDownloader::download_from_link(download_link, filename).await?)
    }
}

#[async_trait]
impl AlbumDownloader for MusifyDownloader {
    async fn download_album(&self, album: Box<dyn Album + Send + Sync>) -> Vec<Result<String, Box<dyn std::error::Error + Send + Sync>>> {
        let re = Regex::new(r"^.*\/(.*\.mp3)$").unwrap();
        let page_link = MusifyDownloader::get_page_link(album.get_name(), PageType::Album).await.unwrap();
        let downloads = MusifyDownloader::get_links_from_album_page(page_link).await.unwrap()
            .iter()
            .map(|f| (f.to_owned(), re.captures(&f).unwrap().get(1).unwrap().as_str().to_string()))
            .collect::<Vec<(String, String)>>();
        let mut res = vec![];
        for download in downloads {
            res.push(MusifyDownloader::download_from_link(download.0, download.1).await);
        }
        res
    }
}

impl Downloader for MusifyDownloader {

}
