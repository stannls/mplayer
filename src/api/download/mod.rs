pub mod musify_downloader;
pub mod download_pool;
pub mod file_sorter;

use async_trait::async_trait;
use musicbrainz_rs::entity::recording::Recording;

#[async_trait]
pub trait AudioDownloader {
    async fn download_song(&self, recording: Recording) ->  Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

