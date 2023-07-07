use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use super::{Album, Artist, Song};
use crate::api::player::SongInfo;
use audiotags::Tag;
use chrono::Duration;
use dirs::audio_dir;
use itertools::Itertools;

pub struct FsScanner {
    artists: Option<Vec<Box<dyn Artist>>>,
}

impl FsScanner {
    pub fn new() -> FsScanner {
        FsScanner { artists: None }
    }
    pub fn get_artists(&mut self) -> Vec<Box<dyn Artist>> {
        if self.artists.is_some() {
            self.artists.clone().unwrap()
        } else {
            self.artists = Some(FsScanner::scan_files());
            self.artists.clone().unwrap()
        }
    }
    fn scan_files() -> Vec<Box<dyn Artist>> {
        let mut dir = audio_dir().unwrap();
        dir.push("mplayer");

        let mut artists = depth_first_search_files(
            fs::read_dir(dir)
                .unwrap()
                .filter(|f| f.is_ok())
                .map(|f| f.unwrap())
                .collect(),
        )
        .into_iter()
        .map(|f| Box::new(FsSong::new(f.path()).unwrap()) as Box<dyn Song + Send + Sync>)
        .group_by(|f| f.get_album_name())
        .into_iter()
        .map(|f| Box::new(FsAlbum::new_2(f.1.collect_vec())) as Box<dyn Album + Send + Sync>)
        .group_by(|f| f.get_songs()[0].get_artist_name())
        .into_iter()
        .map(|f| Box::new(FsArtist::new_2(f.1.collect_vec(), f.0)) as Box<dyn Artist>)
        .collect_vec();
        artists.sort_by_key(|a| a.get_name().to_lowercase());

        artists
    }
}

fn depth_first_search_files(files: Vec<DirEntry>) -> Vec<DirEntry> {
    files
        .into_iter()
        .flat_map(|f| {
            if f.file_type().unwrap().is_dir() {
                depth_first_search_files(
                    fs::read_dir(f.path())
                        .unwrap()
                        .filter(|f| f.is_ok())
                        .map(|f| f.unwrap())
                        .collect(),
                )
            } else {
                vec![f]
            }
        })
        .collect()
}

#[derive(Clone)]
pub struct FsArtist {
    name: String,
    albums: Vec<Box<dyn super::Album + Send + Sync>>,
}

impl FsArtist {
    pub fn new_2(mut albums: Vec<Box<dyn Album + Send + Sync>>, name: String) -> FsArtist {
        albums.sort_by_key(|f| f.get_release_date());
        FsArtist { name, albums }
    }
}

impl Artist for FsArtist {
    fn get_albums(&self) -> Vec<Box<dyn super::Album + Send + Sync>> {
        self.albums.to_owned()
    }

    fn get_name(&self) -> String {
        self.name.to_owned()
    }
}

#[derive(Clone)]
pub struct FsAlbum {
    songs: Vec<Box<dyn super::Song + Send + Sync>>,
}

impl FsAlbum {
    pub fn new(path: PathBuf) -> Option<FsAlbum> {
        let mut songs: Vec<Box<dyn super::Song + Send + Sync>> = fs::read_dir(path.to_owned())
            .unwrap()
            .into_iter()
            .map(|f| {
                Box::new(FsSong::new(f.unwrap().path()).unwrap()) as Box<dyn Song + Send + Sync>
            })
            .collect();
        songs.sort_by(|a, b| {
            a.get_number()
                .unwrap()
                .parse::<isize>()
                .unwrap()
                .partial_cmp(&b.get_number().unwrap().parse::<isize>().unwrap())
                .unwrap()
        });

        Some(FsAlbum { songs })
    }
    pub fn new_2(mut songs: Vec<Box<dyn Song + Send + Sync>>) -> FsAlbum {
        songs.sort_by_key(|f| f.get_number());
        FsAlbum { songs }
    }
}

impl Album for FsAlbum {
    fn get_name(&self) -> String {
        self.songs
            .to_owned()
            .into_iter()
            .map(|f| f.get_album_name())
            .collect::<Vec<String>>()
            .get(0)
            .unwrap_or(&"".to_string())
            .to_owned()
    }

    fn get_release_date(&self) -> String {
        self.songs
            .to_owned()
            .into_iter()
            .map(|f| f.get_release_date())
            .filter(|f| f.is_some())
            .collect::<Vec<Option<String>>>()
            .get(0)
            .unwrap_or(&None)
            .to_owned()
            .unwrap_or("".to_string())
    }

    fn get_songs(&self) -> Vec<Box<dyn super::Song + Send + Sync>> {
        self.songs.to_owned()
    }

    fn is_groups(&self) -> bool {
        false
    }

    fn get_id(&self) -> String {
        "".to_string()
    }

    fn is_local(&self) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct FsSong {
    path: PathBuf,
    title: String,
    length: f64,
    number: u16,
    album_name: String,
    release_data: String,
}

impl FsSong {
    pub fn new(path: PathBuf) -> Option<FsSong> {
        let tags = Tag::new().read_from_path(path.to_owned()).unwrap();
        Some(FsSong {
            path: path.to_owned(),
            title: tags.title().unwrap().to_string(),
            length: mp3_duration::from_path(&path)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_millis() as f64,
            number: tags.track_number().unwrap(),
            album_name: tags.album_title().unwrap_or("").to_string(),
            release_data: tags.year().unwrap_or(0).to_string(),
        })
    }
}

impl Song for FsSong {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }

    fn get_length(&self) -> Option<String> {
        let t = Duration::milliseconds(self.length as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        Some(format!("{:0>2}:{:0>2}", minutes, seconds))
    }

    fn get_length_secs(&self) -> Option<usize> {
        let t = Duration::milliseconds(self.length as i64);
        Some(t.num_seconds() as usize)
    }

    fn get_disambiguation(&self) -> Option<String> {
        None
    }

    fn get_artist_name(&self) -> String {
        self.path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn get_number(&self) -> Option<String> {
        Some(self.number.to_string())
    }

    fn is_local(&self) -> bool {
        true
    }

    fn get_filepath(&self) -> Option<PathBuf> {
        Some(self.path.to_owned())
    }

    fn get_album_name(&self) -> String {
        self.album_name.to_owned()
    }

    fn get_release_date(&self) -> Option<String> {
        if self.release_data == "0" {
            None
        } else {
            Some(self.release_data.to_owned())
        }
    }
}

// Tries to find the album of the given song in the local files
pub fn find_current_album(song_info: &SongInfo) -> Option<Box<dyn Album + Send + Sync>> {
    let mut dir = audio_dir().unwrap();
    dir.push("mplayer");
    let artists = fs::read_dir(dir.clone())
        .unwrap()
        .into_iter()
        .filter(|f| f.is_ok())
        .map(|f| f.unwrap())
        .filter(|f| f.file_name().into_string().unwrap() == song_info.artist)
        .collect::<Vec<DirEntry>>();
    dir.push(artists.get(0)?.file_name());
    let albums = fs::read_dir(dir.clone())
        .unwrap()
        .into_iter()
        .filter(|f| f.is_ok())
        .map(|f| f.unwrap())
        .filter(|f| f.file_name().into_string().unwrap() == song_info.album)
        .collect::<Vec<DirEntry>>();
    Some(Box::new(FsAlbum::new(albums.get(0)?.path())?))
}
