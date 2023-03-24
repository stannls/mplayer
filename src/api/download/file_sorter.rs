use audiotags::Tag;
use dirs::audio_dir;
use std::{fs, path::Path};

#[derive(Clone)]
pub struct FileSorter {}

impl FileSorter {
    pub fn new() -> FileSorter {
        FileSorter {}
    }
    pub fn move_file(
        &self,
        filename: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tags = Tag::new().read_from_path(&filename)?;
        let old_file = Path::new(&filename);
        let mut file_dir = audio_dir().unwrap();
        file_dir.push("mplayer");
        file_dir.push(tags.artist().unwrap());
        file_dir.push(tags.album().unwrap().title);
        fs::create_dir_all(file_dir.to_str().unwrap())?;
        file_dir.push(old_file.file_name().unwrap().to_str().unwrap());
        // Using copy and delete here instead of rename because rename will fail when the
        // destination is on a different mount
        fs::copy(old_file, file_dir)?;
        fs::remove_file(old_file)?;
        Ok(())
    }
}
