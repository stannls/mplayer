pub mod bandcamp_downloader;
pub mod download_pool;
pub mod file_sorter;
pub mod musify_downloader;
use super::{Album, Song};

pub trait SongProvider {
    fn provide_song(
        &self,
        recording: Box<dyn Song>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait AlbumProvider {
    fn provide_album(
        &self,
        album: Box<dyn Album + Send + Sync>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait Downloader: SongProvider + AlbumProvider {}
