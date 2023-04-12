use std::fs;

use dirs::audio_dir;

pub fn scan_artists() -> Vec<String> {
    let mut dir = audio_dir().unwrap();
    dir.push("mplayer");
    let mut artists: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .map(|f| f.unwrap().file_name().into_string().unwrap())
        .collect();
    artists.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    artists
}
