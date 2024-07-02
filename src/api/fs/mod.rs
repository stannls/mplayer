use std::{
    fs::{self, File},
    io::{self, Cursor, Read},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use super::{Album, Artist, Deleteable, Song};
use crate::api::player::SongInfo;
use audiotags::Tag;
use chrono::Duration;
use dirs::cache_dir;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MusicRepository {
    path: PathBuf,
    artists: Arc<Mutex<Vec<Box<dyn Artist + Send + Sync>>>>,
}

impl MusicRepository {
    pub fn new(path: PathBuf) -> MusicRepository {
        MusicRepository {
            path,
            artists: Arc::new(Mutex::new(vec![])),
        }
    }
    fn scan_repository(path: PathBuf) -> Vec<Box<dyn Artist + Send + Sync>> {
        let files = depth_first_search_files(vec![path]);
        files
            .into_iter()
            // Convert filesystem path into song struct containing metadata
            .map(|f| FsSong::new(f))
            // Ignore invalid files and convert valid ones into Song trait
            .filter(|f| f.is_some())
            .map(|f| Box::new(f.unwrap()) as Box<dyn Song + Send + Sync>)
            // Group the songs by their album name
            .chunk_by(|f| f.get_album_name())
            .into_iter()
            // Create an FsAlbum from the grouped song and convert it into an Album trait
            .map(|f| Box::new(FsAlbum::new(f.1.unique_by(|song| song.get_title()).collect_vec())) as Box<dyn Album + Send + Sync>)
            // Group the albums by the album artist name
            .chunk_by(|f| f.get_songs()[0].get_artist_name())
            .into_iter()
            // Create an FsArtist from the grouped albums and convert it into an Artist trait
            .map(|f| {
                Box::new(FsArtist::new_2(f.1.collect_vec(), f.0)) as Box<dyn Artist + Send + Sync>
            })
            .sorted_by_key(|a| a.get_name().to_lowercase())
            .collect_vec()
    }
    pub fn remove_artist(&mut self, artist: Box<dyn Artist + Send + Sync>) {
        let cloned = self.clone();
        let mut artists = cloned.artists.lock().unwrap();
        // Search index of the artist to be removed
        let index = artists
            .iter()
            .position(|x| *x.get_name() == artist.get_name())
            .unwrap();
        // Remove the artist
        let mut new_artists = artists.to_owned();
        new_artists.remove(index);
        *artists = new_artists;
        // Force to repopulate cache
        let _ = self.cache_artists();
    }
    pub fn remove_album(&mut self, album: Box<dyn Album + Send + Sync>) {
        let cloned = self.clone();
        let mut artists = cloned.artists.lock().unwrap();
        let artist_index = artists
            .iter()
            .position(|x| *x.get_name() == album.get_songs().get(0).unwrap().get_artist_name())
            .unwrap();

        let artist = artists.get(artist_index).unwrap();

        let new_artist = Box::new(FsArtist::new_2(
            artist
                .get_albums()
                .iter()
                .filter(|x| x.get_name() != album.get_name())
                .map(|f| f.to_owned())
                .collect(),
            artist.get_name(),
        )) as Box<dyn Artist + Send + Sync>;

        let mut new_artists = artists.to_owned();
        new_artists[artist_index] = new_artist;
        *artists = new_artists;
        let _ = self.cache_artists();
    }
    pub fn remove_song(&mut self, song: Box<dyn Song + Send + Sync>) {
        let cloned = self.clone();
        let mut artists = cloned.artists.lock().unwrap();
        let artist_index = artists
            .iter()
            .position(|x| *x.get_name() == song.get_artist_name())
            .unwrap();

        let artist = artists.get(artist_index).unwrap();
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

        let new_album = Box::new(FsAlbum::new(new_songs)) as Box<dyn Album + Send + Sync>;
        let mut new_albums = artist.get_albums();
        new_albums[album_index] = new_album;
        let new_artist = Box::new(FsArtist::new_2(new_albums, artist.get_name()));
        let mut new_artists = artists.to_owned();
        new_artists[artist_index] = new_artist;
        *artists = new_artists;
        let _ = self.cache_artists();
    }
    pub fn get_artists(&mut self) -> Vec<Box<dyn Artist + Send + Sync>> {
        self.artists.lock().unwrap().clone()
    }
    pub fn cache_artists(&mut self) -> Result<(), io::Error> {
        let artists = self.artists.lock().unwrap().clone();

        let data = serde_json::to_string(
            &artists
                .into_iter()
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
                                    release_data: f.get_release_date().unwrap_or("0".to_string()),
                                    album_name: f.get_album_name(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect::<Vec<SaveableArtist>>(),
        )?;

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
    pub fn load_cached_artists(&mut self) -> Result<(), io::Error> {
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
        let artists = cached_artists
            .into_iter()
            .map(|f| {
                Box::new(FsArtist::new_2(
                    f.albums
                        .into_iter()
                        .map(|f| {
                            Box::new(FsAlbum::new(
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
            .collect();
        let mut guard = self.artists.lock().unwrap();
        *guard = artists;
        Ok(())
    }
    pub fn watch_files(&self) {
        let artists = self.artists.clone();
        let path = self.path.clone();
        thread::spawn(move || loop {
            let scanned_artists = MusicRepository::scan_repository(path.to_owned());
            let mut artist_lock = artists.lock().unwrap();
            *artist_lock = scanned_artists;
        });
    }
    pub fn find_current_album(&self, song_info: &SongInfo) -> Option<Box<dyn Album + Send + Sync>> {
        self.artists
            .lock()
            .unwrap()
            .iter()
            .filter(|artist| artist.get_name() == song_info.artist)
            .nth(0)?
            .get_albums()
            .iter()
            .filter(|album| album.get_name() == song_info.album)
            .nth(0)
            .map(|val| val.to_owned())
    }
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

fn depth_first_search_files(files: Vec<PathBuf>) -> Vec<PathBuf> {
    files
        .into_iter()
        .flat_map(|f| {
            if f.is_dir() {
                depth_first_search_files(
                    fs::read_dir(&f)
                        .unwrap()
                        .map(|d| d.unwrap().path())
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
    pub fn new(mut songs: Vec<Box<dyn Song + Send + Sync>>) -> FsAlbum {
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
    fn get_artist_name(&self) -> String {
        self.songs
            .to_owned()
            .into_iter()
            .map(|f| f.get_artist_name())
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
        let extension = infer::get_from_path(path.to_owned()).ok()??.extension();
        if !(extension == "mp3" || extension == "flac" || extension == "wav" || extension == "m4a")
        {
            return None;
        }
        let tags = Tag::new().read_from_path(path.to_owned()).ok()?;
        Some(FsSong {
            path: path.to_owned(),
            title: tags.title()?.to_string(),
            length: mp3_duration::from_path(&path)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_millis() as f64,
            number: tags.track_number()?,
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
