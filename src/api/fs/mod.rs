use std::{
    fs::{self, DirEntry, File},
    io::{self, Cursor, Read},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::{Album, Artist, Deleteable, Song};
use crate::api::player::SongInfo;
use audiotags::Tag;
use chrono::Duration;
use dirs::{audio_dir, cache_dir};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct FsScanner {
    artists: Arc<Mutex<Option<Vec<Box<dyn Artist + Send + Sync>>>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveableSong {
    path: PathBuf,
    title: String,
    length: f64,
    number: u16,
    album_name: String,
    release_data: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveableAlbum {
    songs: Vec<SaveableSong>,
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveableArtist {
    albums: Vec<SaveableAlbum>,
    name: String,
}

pub fn time_to_millis(time: String) -> Option<f64> {
    let re = Regex::new(r"(\d\d)[:](\d\d)").unwrap();
    let captures = re.captures(&time)?;
    Some(captures[1].parse::<f64>().ok()? * 60000.0 + captures[2].parse::<f64>().ok()? * 1000.0)
}

impl FsScanner {
    pub fn new() -> FsScanner {
        let artists =
            Arc::new(Mutex::new(None)) as Arc<Mutex<Option<Vec<Box<dyn Artist + Sync + Send>>>>>;
        FsScanner::start(artists.to_owned());
        FsScanner { artists }
    }
    pub fn remove_artist(&mut self, artist: Box<dyn Artist + Send + Sync>) {
        let mut artists = self.artists.lock().unwrap();
        match artists.to_owned() {
            Some(a) => {
                let index = a
                    .iter()
                    .position(|x| *x.get_name() == artist.get_name())
                    .unwrap();
                let mut new_artists = artists.to_owned().unwrap();
                new_artists.remove(index);
                *artists = Some(new_artists);
                let _ = FsScanner::cache_artists(artists.to_owned().unwrap());
            }
            None => {}
        }
    }
    pub fn remove_album(&mut self, album: Box<dyn Album + Send + Sync>) {
        let mut artists = self.artists.lock().unwrap();
        match artists.to_owned() {
            Some(a) => {
                let artist_index = a
                    .iter()
                    .position(|x| {
                        *x.get_name() == album.get_songs().get(0).unwrap().get_artist_name()
                    })
                    .unwrap();
                let artist = a.get(artist_index).unwrap();
                let new_artist = Box::new(FsArtist::new_2(
                    artist
                        .get_albums()
                        .iter()
                        .filter(|x| x.get_name() != album.get_name())
                        .map(|f| f.to_owned())
                        .collect(),
                    artist.get_name(),
                )) as Box<dyn Artist + Send + Sync>;
                let mut new_artists = a.to_owned();
                new_artists[artist_index] = new_artist;
                *artists = Some(new_artists);
                let _ = FsScanner::cache_artists(artists.to_owned().unwrap());
            }
            None => {}
        }
    }
    pub fn remove_song(&mut self, song: Box<dyn Song + Send + Sync>) {
        let mut artists = self.artists.lock().unwrap();
        match artists.to_owned() {
            Some(a) => {
                let artist_index = a
                    .iter()
                    .position(|x| *x.get_name() == song.get_artist_name())
                    .unwrap();
                let artist = a.get(artist_index).unwrap();
                let album_index = artist
                    .get_albums()
                    .iter()
                    .position(|x| *x.get_name() == song.get_album_name())
                    .unwrap();
                let new_songs = artist
                    .get_albums()
                    .get(album_index)
                    .unwrap()
                    .get_songs()
                    .iter()
                    .filter(|x| x.get_title() != song.get_title())
                    .map(|f| f.to_owned())
                    .collect();
                let new_album = Box::new(FsAlbum::new_2(new_songs)) as Box<dyn Album + Send + Sync>;
                let mut new_albums = artist.get_albums();
                new_albums[album_index] = new_album;
                let new_artist = Box::new(FsArtist::new_2(new_albums, artist.get_name()));
                let mut new_artists = a.to_owned();
                new_artists[artist_index] = new_artist;
                *artists = Some(new_artists);
                let _ = FsScanner::cache_artists(artists.to_owned().unwrap());
            }
            None => {}
        }
    }
    fn start(artists: Arc<Mutex<Option<Vec<Box<dyn Artist + Sync + Send>>>>>) {
        std::thread::spawn(move || {
            let mut guard = artists.lock().unwrap();
            *guard = FsScanner::load_cached_artists().ok();
            drop(guard);
            loop {
                let scanned_artists = FsScanner::scan_files();
                let mut old_artists = artists.lock().unwrap();
                *old_artists = Some(scanned_artists.to_owned());
                let _ = FsScanner::cache_artists(scanned_artists);
            }
        });
    }
    pub fn get_artists(&mut self) -> Vec<Box<dyn Artist + Send + Sync>> {
        let artists = self.artists.lock().unwrap().to_owned();
        artists.unwrap_or(vec![])
    }
    fn scan_files() -> Vec<Box<dyn Artist + Send + Sync>> {
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
        .map(|f| Box::new(FsArtist::new_2(f.1.collect_vec(), f.0)) as Box<dyn Artist + Send + Sync>)
        .collect_vec();
        artists.sort_by_key(|a| a.get_name().to_lowercase());

        artists
    }
    fn artists_to_json(artists: Option<Vec<Box<dyn Artist + Send + Sync>>>) -> Option<String> {
        match artists {
            Some(f) => serde_json::to_string(
                &f.into_iter()
                    .map(|f| SaveableArtist {
                        name: f.get_name(),
                        albums: f
                            .get_albums()
                            .into_iter()
                            .map(|f| SaveableAlbum {
                                name: f.get_name(),
                                songs: f
                                    .get_songs()
                                    .into_iter()
                                    .filter(|f| f.get_filepath().is_some())
                                    .map(|f| SaveableSong {
                                        path: f.get_filepath().unwrap(),
                                        title: f.get_title(),
                                        length: time_to_millis(
                                            f.get_length().unwrap_or("00:00".to_string()),
                                        )
                                        .unwrap_or(0 as f64),
                                        number: f
                                            .get_number()
                                            .unwrap_or("0".to_string())
                                            .parse()
                                            .unwrap_or(0),
                                        release_data: f
                                            .get_release_date()
                                            .unwrap_or("0".to_string()),
                                        album_name: f.get_album_name(),
                                    })
                                    .collect(),
                            })
                            .collect(),
                    })
                    .collect::<Vec<SaveableArtist>>(),
            )
            .ok(),
            _ => None,
        }
    }
    pub fn cache_artists(artists: Vec<Box<dyn Artist + Send + Sync>>) -> Result<(), io::Error> {
        let data = FsScanner::artists_to_json(Some(artists)).unwrap();
        let mut path = cache_dir().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Failed to find config dir",
        ))?;
        path.push("mplayer");
        fs::create_dir_all(&path)?;
        path.push("artist_cache");
        let mut file = File::create(&path)?;
        let mut content = Cursor::new(data);
        io::copy(&mut content, &mut file)?;
        Ok(())
    }
    fn load_cached_artists() -> Result<Vec<Box<dyn Artist + Send + Sync>>, io::Error> {
        let mut path = cache_dir().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Failed to find config dir",
        ))?;
        path.push("mplayer");
        path.push("artist_cache");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let cached_artists: Vec<SaveableArtist> = serde_json::from_str(&contents)?;
        Ok(cached_artists
            .into_iter()
            .map(|f| {
                Box::new(FsArtist::new_2(
                    f.albums
                        .into_iter()
                        .map(|f| {
                            Box::new(FsAlbum::new_2(
                                f.songs
                                    .into_iter()
                                    .map(|f| {
                                        Box::new(FsSong::fastnew(
                                            f.path,
                                            f.title,
                                            f.length,
                                            f.number,
                                            f.album_name,
                                            f.release_data,
                                        ))
                                            as Box<dyn Song + Send + Sync>
                                    })
                                    .collect(),
                            )) as Box<dyn Album + Send + Sync>
                        })
                        .collect(),
                    f.name,
                )) as Box<dyn Artist + Send + Sync>
            })
            .collect())
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
    fn is_local(&self) -> bool {
        true
    }
}

impl Deleteable for FsArtist {
    fn delete(&self) {
        for album in &self.albums {
            album.delete();
        }
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
        songs.sort_by(|a, b| {
            a.get_number()
                .unwrap()
                .parse::<isize>()
                .unwrap()
                .partial_cmp(&b.get_number().unwrap().parse::<isize>().unwrap())
                .unwrap()
        });
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

impl Deleteable for FsAlbum {
    fn delete(&self) {
        for song in &self.songs {
            song.delete();
        }
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
    pub fn fastnew(
        path: PathBuf,
        title: String,
        length: f64,
        number: u16,
        album_name: String,
        release_data: String,
    ) -> FsSong {
        FsSong {
            path,
            title,
            length,
            number,
            album_name,
            release_data,
        }
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

impl Deleteable for FsSong {
    fn delete(&self) {
        let _ = fs::remove_file(&self.path);
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
