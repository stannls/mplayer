use crate::api::{Album, Song};

use super::file_sorter::FileSorter;
use super::Downloader;
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;
use threadpool::ThreadPool;
use std::panic;


#[derive(Clone)]
pub struct DownloadPool {
    downloaders: Vec<Arc<dyn Downloader + Sync + Send>>,
    threadpool: ThreadPool,
    file_sorter: FileSorter,
}

impl DownloadPool {
    pub fn new(thread_count: usize) -> DownloadPool {
        DownloadPool {
            downloaders: vec![],
            threadpool: ThreadPool::new(thread_count),
            file_sorter: FileSorter::new(),
        }
    }
    pub fn add_downloader(mut self, downloader: Arc<dyn Downloader + Send + Sync>) -> DownloadPool {
        self.downloaders.push(downloader);
        self
    }
    pub fn download_song(&self, recording: Box<dyn Song>) {
        let fs = Arc::new(self.file_sorter.clone());
        let download_link = self.try_all_song_providers(recording.clone());
        if download_link.is_some() {
            self.threadpool.execute(move || {
                panic::set_hook(Box::new(|_info| {
                    // do nothing
                }));
                let filepath = DownloadPool::download_from_link(
                    download_link.unwrap(),
                    format!("{}-{}", recording.get_artist_name(), recording.get_title()),
                )
                .unwrap();
                fs.move_and_tag_file(filepath, recording).unwrap();
            })
        }
    }
    pub fn download_album(&self, recording: Box<dyn Album + Send + Sync>) {
        let download_links = self.try_all_album_providers(recording.clone());
        if download_links.is_some() {
            for i in 0..download_links.as_ref().unwrap().len() {
                let current_song = recording.get_songs()[i].clone();
                let current_link = download_links.clone().unwrap()[i].clone();
                let fs = Arc::new(self.file_sorter.clone());

                self.threadpool.execute(move || {
                    let filepath = DownloadPool::download_from_link(
                        current_link,
                        format!(
                            "{}-{}",
                            current_song.get_artist_name(),
                            current_song.get_title()
                        ),
                    )
                    .unwrap();
                    fs.move_and_tag_file(filepath, current_song).unwrap();
                })
            }
        }
    }
    fn download_from_link(
        link: String,
        filename: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::blocking::get(link)?.bytes()?;
        let path = format!("/tmp/{}.mp3", filename);
        let mut file = File::create(&path)?;
        let mut content = Cursor::new(response);
        std::io::copy(&mut content, &mut file)?;
        Ok(path)
    }
    fn try_all_song_providers(&self, recording: Box<dyn Song>) -> Option<String> {
        for downloader in self.downloaders.clone() {
            match downloader.provide_song(recording.to_owned()) {
                Ok(filename) => return Some(filename),
                _ => {}
            }
        }
        return None;
    }
    fn try_all_album_providers(&self, album: Box<dyn Album + Send + Sync>) -> Option<Vec<String>> {
        let mut provided = vec![];
        for downloader in self.downloaders.clone() {
            let filenames = downloader.provide_album(album.to_owned());
            match filenames {
                Ok(filenames) => if filenames.len() == album.get_songs().len() {
                    return Some(filenames);
                } else {
                    provided.push(filenames);
                }
                _ => {}
            }
        }
        return if provided.len() > 0 {
            provided.sort_by_key(|filenames| filenames.len());
            provided.reverse();
            Some(provided[0].to_owned())
        } else {    
            return None;
        }
    }
}
unsafe impl Send for DownloadPool {}
unsafe impl Sync for DownloadPool {}
