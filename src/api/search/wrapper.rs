use crate::api::{Album, Artist, Song};
use chrono::Duration;
use musicbrainz_rs::entity::{
    artist, recording,
    release::{Release, Track},
    release_group::{self, ReleaseGroup},
};

#[derive(Clone)]
pub struct ArtistWrapper {
    pub data: artist::Artist,
    release_groups: Vec<ReleaseGroup>,
}

impl ArtistWrapper {
    pub fn new(artist: artist::Artist, release_groups: Vec<ReleaseGroup>) -> ArtistWrapper {
        ArtistWrapper {
            data: artist,
            release_groups,
        }
    }
    pub fn releases(&self, release_groups: Vec<ReleaseGroup>) -> ArtistWrapper {
        ArtistWrapper {
            data: self.data.to_owned(),
            release_groups,
        }
    }
}

impl SearchEntity for ArtistWrapper {
    fn display(&self) -> String {
        self.data.name.to_owned()
    }
}

impl Artist for ArtistWrapper {
    fn get_name(&self) -> String {
        self.data.name.to_owned()
    }
    fn get_albums(&self) -> Vec<Box<dyn Album + Send + Sync>> {
        self.release_groups
            .to_owned()
            .into_iter()
            .map(|f| Box::new(ReleaseGroupWrapper::new(f)) as Box<dyn Album + Send + Sync>)
            .collect()
    }
}

#[derive(Clone)]
pub struct SongWrapper {
    pub data: recording::Recording,
    album_name: String,
}

impl SongWrapper {
    pub fn new(recording: recording::Recording, album_name: String) -> SongWrapper {
        SongWrapper {
            data: recording,
            album_name,
        }
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
    fn get_length(&self) -> Option<String> {
        let t = Duration::milliseconds(self.data.length? as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        Some(format!("{:0>2}:{:0>2}", minutes, seconds))
    }

    fn is_local(&self) -> bool {
        false
    }

    fn get_filepath(&self) -> Option<std::path::PathBuf> {
        None
    }

    fn get_album_name(&self) -> String {
        self.album_name.to_owned()
    }

    fn get_length_secs(&self) -> Option<usize> {
        let t = Duration::milliseconds(self.data.length? as i64);
        Some(t.num_seconds() as usize)
    }

    fn get_release_date(&self) -> Option<String> {
        Some(self.data.releases.to_owned()?.get(0)?.date?.to_string())
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

impl Album for ReleaseGroupWrapper {
    fn get_name(&self) -> String {
        self.data.title.to_owned()
    }
    fn get_release_date(&self) -> String {
        self.data.first_release_date.unwrap().to_string()
    }
    fn get_songs(&self) -> Vec<Box<dyn Song + Send + Sync>> {
        vec![]
    }
    fn is_groups(&self) -> bool {
        true
    }
    fn get_id(&self) -> String {
        self.data.id.to_owned()
    }

    fn is_local(&self) -> bool {
        false
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
    fn get_songs(&self) -> Vec<Box<dyn Song + Send + Sync>> {
        let media = self.data.media.to_owned().unwrap();
        for m in media.clone() {
            if m.format.to_owned().unwrap() == "CD" {
                let mut songs: Vec<Box<dyn Song + Send + Sync>> = vec![];
                for s in media.get(0).unwrap().tracks.to_owned().unwrap() {
                    songs.push(Box::new(TrackWrapper::new(s, self.get_name())));
                }
                return songs;
            }
        }
        let mut songs: Vec<Box<dyn Song + Send + Sync>> = vec![];
        for s in media.get(0).unwrap().tracks.to_owned().unwrap() {
            songs.push(Box::new(TrackWrapper::new(s, self.get_name())));
        }
        songs
    }
    fn is_groups(&self) -> bool {
        false
    }
    fn get_id(&self) -> String {
        self.data.id.to_owned()
    }

    fn is_local(&self) -> bool {
        false
    }
}

pub trait SearchEntity {
    fn display(&self) -> String;
}

#[derive(Clone)]
pub struct TrackWrapper {
    data: Track,
    // Needs to be stored here, because else there is no easy way to access the album title
    album_name: String,
}

impl TrackWrapper {
    pub fn new(d: Track, album_name: String) -> TrackWrapper {
        TrackWrapper {
            data: d,
            album_name,
        }
    }
}

impl Song for TrackWrapper {
    fn get_title(&self) -> String {
        self.data.title.to_owned()
    }
    fn get_length(&self) -> Option<String> {
        let t = Duration::milliseconds(self.data.length? as i64);
        let seconds = t.num_seconds() % 60;
        let minutes = (t.num_seconds() / 60) % 60;
        Some(format!("{:0>2}:{:0>2}", minutes, seconds))
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

    fn is_local(&self) -> bool {
        false
    }

    fn get_filepath(&self) -> Option<std::path::PathBuf> {
        None
    }

    fn get_album_name(&self) -> String {
        self.album_name.to_owned()
    }

    fn get_length_secs(&self) -> Option<usize> {
        let t = Duration::milliseconds(self.data.length? as i64);
        Some(t.num_seconds() as usize)
    }

    fn get_release_date(&self) -> Option<String> {
        None
    }
}
