use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::api::player::SongInfo;
use audiotags::Tag;
use chrono::Duration;
use dirs::audio_dir;

use super::{Album, Artist, Song};

pub fn scan_artists() -> Vec<String> {
    let mut dir = audio_dir().unwrap();
    dir.push("mplayer");
    fs::create_dir_all(&dir).unwrap();
    let mut artists: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .map(|f| f.unwrap().file_name().into_string().unwrap())
        .collect();
    artists.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    artists
}

#[derive(Clone)]
pub struct FsArtist {
    path: PathBuf,
    albums: Vec<Box<dyn super::Album + Send + Sync>>,
}

impl FsArtist {
    pub fn new(name: String) -> Option<FsArtist> {
        let mut path = audio_dir().unwrap();
        path.push("mplayer");
        path.push(name);
        let mut albums: Vec<Box<dyn super::Album + Send + Sync>> = fs::read_dir(path.to_owned())
            .unwrap()
            .into_iter()
            .map(|f| {
                Box::new(FsAlbum::new(f.unwrap().path()).unwrap()) as Box<dyn Album + Send + Sync>
            })
            .collect();
        albums.sort_by(|a, b| {
            a.get_release_date()
                .parse::<usize>()
                .unwrap_or(0)
                .partial_cmp(&b.get_release_date().parse::<usize>().unwrap_or(0))
                .unwrap()
        });
        albums.reverse();
        Some(FsArtist { path, albums })
    }
}

impl Artist for FsArtist {
    fn get_albums(&self) -> Vec<Box<dyn super::Album + Send + Sync>> {
        self.albums.to_owned()
    }

    fn get_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

#[derive(Clone)]
pub struct FsAlbum {
    path: PathBuf,
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

        Some(FsAlbum { path, songs })
    }
}

impl Album for FsAlbum {
    fn get_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
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
