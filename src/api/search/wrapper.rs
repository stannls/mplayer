use musicbrainz_rs::entity::{artist, recording, release_group};

#[derive(Clone)]
pub struct Artist {
    pub data: artist::Artist,
}

impl Artist {
    pub fn new(artist: artist::Artist) -> Artist {
        Artist { data: artist }
    }
}

impl SearchEntity for Artist {
    fn display(&self) -> String {
        self.data.name.to_owned()
    }
}

#[derive(Clone)]
pub struct Recording {
    pub data: recording::Recording,
}

impl Recording {
    pub fn new(recording: recording::Recording) -> Recording {
        Recording { data: recording }
    }
}

impl SearchEntity for Recording {
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

#[derive(Clone)]
pub struct Release {
    pub data: release_group::ReleaseGroup,
}

impl Release {
    pub fn new(release: release_group::ReleaseGroup) -> Release {
        Release { data: release }
    }
}

impl SearchEntity for Release {
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

pub trait SearchEntity {
    fn display(&self) -> String;
}
