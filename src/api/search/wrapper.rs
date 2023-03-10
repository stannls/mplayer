use musicbrainz_rs::entity::{release, recording, artist};

#[derive(Clone)]
pub struct Artist{
    data: artist::Artist,
}

impl Artist {
    pub fn new(artist: artist::Artist) -> Artist{
        Artist { data: artist }
    }
}

impl SearchEntity for Artist {
    fn display(&self) -> String {
        self.data.name.to_owned()
    }
}

#[derive(Clone)]
pub struct Recording{
    data: recording::Recording,
}

impl Recording {
    pub fn new(recording: recording::Recording) -> Recording{
        Recording { data: recording }
    }
}

impl SearchEntity for Recording {
   fn display(&self) -> String {
       format!("{} - {}", self.data.title, self.data.artist_credit.to_owned().unwrap().get(0).unwrap().name)
   } 
}

#[derive(Clone)]
pub struct Release{
    data: release::Release,
}

impl Release {
    pub fn new(release: release::Release) -> Release{
        Release {data: release}
    }
}

impl SearchEntity for Release {
    fn display(&self) -> String {
        format!("{} - {}", self.data.title, self.data.artist_credit.to_owned().unwrap().get(0).unwrap().name)
    }
}

pub trait SearchEntity {
    fn display(&self) -> String;
}
