use std::{fs, path::PathBuf};

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
    albums: Vec<Box<dyn super::Album>>,
}

impl FsArtist {
    pub fn new(name: String) -> Option<FsArtist> {
        let mut path = audio_dir().unwrap();
        path.push("mplayer");
        path.push(name);

        let albums = fs::read_dir(path.to_owned())
            .unwrap()
            .into_iter()
            .map(|f| Box::new(FsAlbum::new(f.unwrap().path()).unwrap()) as Box<dyn Album>)
            .collect();
        Some(FsArtist { path, albums })
    }
}

impl Artist for FsArtist {
    fn get_albums(&self) -> Vec<Box<dyn super::Album>> {
        self.albums.to_owned()
    }

    fn get_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

#[derive(Clone)]
pub struct FsAlbum {
    path: PathBuf,
    songs: Vec<Box<dyn super::Song>>,
}

impl FsAlbum {
    pub fn new(path: PathBuf) -> Option<FsAlbum> {
        let mut songs: Vec<Box<dyn super::Song>> = fs::read_dir(path.to_owned())
            .unwrap()
            .into_iter()
            .map(|f| Box::new(FsSong::new(f.unwrap().path()).unwrap()) as Box<dyn Song>)
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
        "".to_string()
    }

    fn get_songs(&self) -> Vec<Box<dyn super::Song>> {
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
}

impl FsSong {
    pub fn new(path: PathBuf) -> Option<FsSong> {
        let tags = Tag::new().read_from_path(path.to_owned()).unwrap();
        Some(FsSong {
            path: path.to_owned(),
            title: tags.title().unwrap().to_string(),
            length: mp3_duration::from_path(&path).unwrap().as_millis() as f64,
            number: tags.track_number().unwrap(),
            album_name: tags.album_title().unwrap_or("").to_string(),
        })
    }
}

impl Song for FsSong {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }

    fn get_length(&self) -> String {
        let t = Duration::milliseconds(self.length as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        format!("{:0>2}:{:0>2}", minutes, seconds)
    }

    fn get_length_secs(&self) -> usize {
        let t = Duration::milliseconds(self.length as i64);
        t.num_seconds() as usize
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
}
