use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use super::{Album, Song};

pub struct MusicPlayer {
    // These two need to be stored in the struct, because else they will go out of scope and the
    // sink will be unable to play
    _stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    sink: Sink,
}

impl MusicPlayer {
    pub fn new() -> MusicPlayer {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.75);
        MusicPlayer {
            _stream_handle: stream_handle,
            _stream: stream,
            sink,
        }
    }
    pub fn play_song(&self, song: Box<dyn Song>) {
        match song.get_filepath() {
            Some(filepath) => {
                let file = BufReader::new(File::open(filepath.into_os_string()).unwrap());
                let source = Decoder::new(file).unwrap();
                // Clears queue and starts playback of the current song
                self.sink.stop();
                self.sink.append(source);
            }
            // Trying to play a non local song object will be ignored for now
            None => {}
        }
    }
    pub fn play_album(&self, album: Box<dyn Album>) {
        if album.is_local() {
            self.sink.stop();
            for song in album.get_songs() {
                match song.get_filepath() {
                    Some(filepath) => {
                        let file = BufReader::new(File::open(filepath.into_os_string()).unwrap());
                        let source = Decoder::new(file).unwrap();
                        self.sink.append(source);
                    }
                    // Trying to play a non local song object will be ignored for now
                    None => {}
                }
            }
        }
    }
}
