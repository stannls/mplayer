use crate::api::Song;

use super::file_sorter::FileSorter;
use super::AudioDownloader;

use std::sync::Arc;
use threadpool::ThreadPool;
use tokio::runtime::Runtime;

pub struct DownloadPool {
    downloaders: Vec<Arc<dyn AudioDownloader + Sync + Send>>,
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
    pub fn add_downloader(
        mut self,
        downloader: Arc<dyn AudioDownloader + Send + Sync>,
    ) -> DownloadPool {
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
            fs.move_file(filepath).unwrap();
        });
    }
    pub fn download_songs(&self, recordings: Vec<Box<dyn Song>>) {
        for r in recordings {
            self.download_song(r);
        }
    }
}
