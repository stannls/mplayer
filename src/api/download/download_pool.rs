use crate::api::{Album, Song};

use super::file_sorter::FileSorter;
use super::Downloader;

use std::sync::Arc;
use threadpool::ThreadPool;
use tokio::runtime::Runtime;

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
        let downloaders = self.downloaders.clone();
        let fs = Arc::new(self.file_sorter.clone());
        self.threadpool.execute(move || {
            let try_all_downloaders = async || {
                for downloader in downloaders {
                    match downloader.download_song(recording.to_owned()).await {
                        Ok(filename) => return Ok(filename),
                        _ => {}
                    }
                }
                return Err("No working provider");
            };
            let rt = Runtime::new().unwrap();
            let filepath = rt.block_on(try_all_downloaders()).unwrap();
            fs.move_and_tag_file(filepath, recording).unwrap();
        });
    }
    pub fn download_album(&self, recording: Box<dyn Album + Send + Sync>) {
        let downloaders = self.downloaders.clone();
        let fs = Arc::new(self.file_sorter.clone());
        self.threadpool.execute(move || {
            let get_files = async || {
                downloaders
                    .get(0)
                    .unwrap()
                    .download_album(recording.to_owned())
                    .await
            };
            let rt = Runtime::new().unwrap();
            let files = rt.block_on(get_files());
            for i in 0..recording.get_songs().len() {
                if files.get(i).is_some() && files[i].is_ok() {
                    let _ = fs.move_and_tag_file(
                        files[i].as_ref().unwrap().to_owned(),
                        recording.get_songs()[i].to_owned(),
                    );
                }
            }
        })
    }
}
