use audiotags::{AudioTag, Id3v2Tag, Tag};
use dirs::audio_dir;
use std::{error::Error, fs, path::Path};

use crate::api::Song;

#[derive(Clone)]
pub struct FileSorter {}

impl FileSorter {
    pub fn new() -> FileSorter {
        FileSorter {}
    }
    pub fn move_and_tag_file(
        &self,
        filename: String,
        song: Box<dyn Song>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tags = self.correct_tags(&filename, &song)?;
        tags.write_to_path(&filename)?;

        let old_file = Path::new(&filename);
        let mut file_dir = audio_dir().unwrap();
        file_dir.push("mplayer");
        file_dir.push(song.get_artist_name());
        file_dir.push(song.get_album_name());
        fs::create_dir_all(file_dir.to_str().unwrap())?;
        file_dir.push(old_file.file_name().unwrap().to_str().unwrap());

        // Using copy and delete here instead of rename because rename will fail when the
        // destination is on a different mount
        fs::copy(old_file, file_dir)?;
        fs::remove_file(old_file)?;
        Ok(())
    }
    fn correct_tags(
        &self,
        filepath: &str,
        song: &Box<dyn Song>,
    ) -> Result<Box<dyn AudioTag>, Box<dyn Error + Send + Sync>> {
        let mut tags = Tag::new()
            .read_from_path(&filepath)
            .unwrap_or(Box::new(Id3v2Tag::new()));
        if matches!(tags.title(), Option::None) {
            tags.set_title(&song.get_title());
        }
        if matches!(tags.artist(), Option::None) {
            tags.set_title(&song.get_artist_name());
        }
        if matches!(tags.album_title(), Option::None) {
            tags.set_album_artist(&song.get_album_name());
        }
        if matches!(tags.track_number(), Option::None) {
            tags.set_track_number(song.get_number().unwrap().parse().unwrap());
        }
        Ok(tags)
    }
}
unsafe impl Send for FileSorter {}
unsafe impl Sync for FileSorter {}
