use crate::api::download::AudioDownloader;
use std::sync::Arc;
use std::fs::File;
use serde::Deserialize;
use std::io::Cursor;
use regex::Regex;
use async_trait::async_trait;
use musicbrainz_rs::entity::recording::Recording;

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

    async fn get_page_link(song_title: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let songs = client.get(format!("https://musify.club/search/suggestions?term={}", song_title))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:86.0) Gecko/20100101 Firefox/86.0")
            .header("Referer", "https://musify.club/")
            .send()
            .await?
            .json::<Vec<SearchResult>>().await?
            .into_iter()
            .filter(|f| f.category == "Треки")
            .collect::<Vec<SearchResult>>();
        Ok(songs.get(0).unwrap().url.to_owned())
    }

    fn parse_page_link(page_link: String) ->  Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let re = Regex::new(r"^/track/(.*)-(\d*)$")?;
        let captures = re.captures(&page_link).unwrap();
        Ok((format!("https://musify.club/track/dl/{}/{}.mp3", captures.get(2).unwrap().as_str(), captures.get(1).unwrap().as_str()), String::from(captures.get(1).unwrap().as_str())))
    }

}


#[async_trait]
impl AudioDownloader for MusifyDownloader {
    async fn download_song(&self, recording: Recording) ->  Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("{} - {}", recording.artist_credit.unwrap().get(0).unwrap().name, recording.title);
        let page_link = MusifyDownloader::get_page_link(query).await?;
        let (download_link, filename) = MusifyDownloader::parse_page_link(page_link)?;
        Ok(MusifyDownloader::download_from_link(download_link, filename).await?)
    }
}
