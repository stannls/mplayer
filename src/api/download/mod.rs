pub mod musify_downloader;
pub mod download_pool;
pub mod file_sorter;
pub mod bandcamp_downloader;

use async_trait::async_trait;


use super::{Song, Album};

#[async_trait]
pub trait SongDownloader {
    async fn download_song(&self, recording: Box<dyn Song>) ->  Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait AlbumDownloader {
    async fn download_album(&self, album: Box<dyn Album + Send + Sync>) -> Vec<Result<String, Box<dyn std::error::Error + Send + Sync>>>;
}

pub trait Downloader: SongDownloader + AlbumDownloader {

}
