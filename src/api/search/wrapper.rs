use crate::api::{Album, Song};
use chrono::Duration;
use musicbrainz_rs::entity::{
    artist, recording,
    release::{Release, Track},
    release_group,
};

#[derive(Clone)]
pub struct ArtistWrapper {
    pub data: artist::Artist,
}

impl ArtistWrapper {
    pub fn new(artist: artist::Artist) -> ArtistWrapper {
        ArtistWrapper { data: artist }
    }
}

impl SearchEntity for ArtistWrapper {
    fn display(&self) -> String {
        self.data.name.to_owned()
    }
}

#[derive(Clone)]
pub struct SongWrapper {
    pub data: recording::Recording,
}

impl SongWrapper {
    pub fn new(recording: recording::Recording) -> SongWrapper {
        SongWrapper { data: recording }
    }
}

impl SearchEntity for SongWrapper {
    fn display(&self) -> String {
        let disambiguation = match self.data.disambiguation.clone() {
            Some(s) => format!(" ({})", s),
            _ => format!(""),
        };
        format!(
            "{} - {}{}",
            self.data.title,
            self.data
                .artist_credit
                .to_owned()
                .unwrap()
                .get(0)
                .unwrap()
                .name,
            disambiguation
        )
    }
}

impl Song for SongWrapper {
    fn get_title(&self) -> String {
        self.data.title.to_owned()
    }
    fn get_disambiguation(&self) -> Option<String> {
        self.data.disambiguation.to_owned()
    }
    fn get_number(&self) -> Option<String> {
        None
    }
    fn get_artist_name(&self) -> String {
        self.data
            .artist_credit
            .clone()
            .unwrap()
            .get(0)
            .unwrap()
            .name
            .to_owned()
    }
    fn get_length(&self) -> String {
        let t = Duration::milliseconds(self.data.length.unwrap() as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        format!("{:0>2}:{:0>2}", minutes, seconds)
    }
}

#[derive(Clone)]
pub struct ReleaseGroupWrapper {
    pub data: release_group::ReleaseGroup,
}

impl ReleaseGroupWrapper {
    pub fn new(release: release_group::ReleaseGroup) -> ReleaseGroupWrapper {
        ReleaseGroupWrapper { data: release }
    }
}

impl SearchEntity for ReleaseGroupWrapper {
    fn display(&self) -> String {
        format!(
            "{} - {}",
            self.data.title,
            self.data
                .artist_credit
                .to_owned()
                .unwrap()
                .get(0)
                .unwrap()
                .name
        )
    }
}

#[derive(Clone)]
pub struct AlbumWrapper {
    pub data: Release,
}

impl AlbumWrapper {
    pub fn new(data: Release) -> AlbumWrapper {
        AlbumWrapper { data }
    }
}

impl Album for AlbumWrapper {
    fn get_name(&self) -> String {
        self.data.title.to_owned()
    }
    fn get_release_date(&self) -> String {
        self.data.date.unwrap().to_string()
    }
    fn get_songs(&self) -> Vec<Box<dyn Song>> {
        let media = self.data.media.to_owned().unwrap();
        for m in media.clone() {
            if m.format.to_owned().unwrap() == "CD" {
                let mut songs: Vec<Box<dyn Song>> = vec![];
                for s in media.get(0).unwrap().tracks.to_owned().unwrap() {
                    songs.push(Box::new(TrackWrapper::new(s)));
                }
                return songs;
            }
        }
        let mut songs: Vec<Box<dyn Song>> = vec![];
        for s in media.get(0).unwrap().tracks.to_owned().unwrap() {
            songs.push(Box::new(TrackWrapper::new(s)));
        }
        songs
    }
}

pub trait SearchEntity {
    fn display(&self) -> String;
}

#[derive(Clone)]
pub struct TrackWrapper {
    data: Track,
}

impl TrackWrapper {
    pub fn new(d: Track) -> TrackWrapper {
        TrackWrapper { data: d }
    }
}

impl Song for TrackWrapper {
    fn get_title(&self) -> String {
        self.data.title.to_owned()
    }
    fn get_length(&self) -> String {
        let t = Duration::milliseconds(self.data.length.unwrap() as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        format!("{:0>2}:{:0>2}", minutes, seconds)
    }
    fn get_number(&self) -> Option<String> {
        Some(self.data.number.to_owned())
    }
    fn get_artist_name(&self) -> String {
        self.data
            .recording
            .artist_credit
            .to_owned()
            .unwrap()
            .get(0)
            .unwrap()
            .name
            .to_owned()
    }
    fn get_disambiguation(&self) -> Option<String> {
        None
    }
}
