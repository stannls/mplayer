use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink};

use super::{Album, Song};

// This struct represents all possible interactions with the music player
pub enum MusicPlayerEvent {
    Stop,
    Play(Decoder<BufReader<File>>),
    Skip,
    Pause,
    None,
}

pub struct MusicPlayer {
    // These two need to be stored in the struct, because else they will go out of scope and the
    // sink will be unable to play
    sender: Sender<MusicPlayerEvent>,
}

impl MusicPlayer {
    pub fn new() -> MusicPlayer {
        let (tx, rx) = mpsc::channel::<MusicPlayerEvent>();
        MusicPlayer::start(rx);
        MusicPlayer { sender: tx }
    }
    fn start(rx: Receiver<MusicPlayerEvent>) {
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let mut queue = VecDeque::new();
            loop {
                match rx.try_recv().unwrap_or(MusicPlayerEvent::None) {
                    MusicPlayerEvent::Play(source) => {
                        sink.play();
                        if sink.empty() {
                            sink.append(source);
                        } else {
                            queue.push_back(source)
                        }
                    }
                    MusicPlayerEvent::Stop => {
                        sink.stop();
                        queue.clear();
                    }
                    MusicPlayerEvent::Skip => {
                        if !sink.empty() && !queue.is_empty() {
                            sink.stop();
                            sink.append(queue.pop_front().unwrap());
                        }
                    }
                    MusicPlayerEvent::Pause => {
                        if sink.is_paused() {
                            sink.play()
                        } else {
                            sink.pause()
                        }
                    }
                    MusicPlayerEvent::None => {}
                }
                // Event for playing a new song after the last is finished
                if !queue.is_empty() && sink.empty() {
                    sink.append(queue.pop_front().unwrap());
                }
                thread::sleep(Duration::from_millis(50))
            }
        });
    }
    // Emptys queue, plays song
    pub fn play_song(&self, song: Box<dyn Song>) {
        let file = BufReader::new(File::open(song.get_filepath().unwrap()).unwrap());
        let source = Decoder::new(file).unwrap();
        self.stop();
        self.sender.send(MusicPlayerEvent::Play(source)).unwrap();
    }
    // Emptys queue, enqueues album
    pub fn play_album(&self, album: Box<dyn Album>) {
        self.stop();
        for song in album.get_songs() {
            let file = BufReader::new(File::open(song.get_filepath().unwrap()).unwrap());
            let source = Decoder::new(file).unwrap();
            self.sender.send(MusicPlayerEvent::Play(source)).unwrap();
        }
    }
    // Pauses if playing, continues if paused
    pub fn pause(&self) {
        self.sender.send(MusicPlayerEvent::Pause).unwrap();
    }
    // Skips one song
    pub fn skip(&self) {
        self.sender.send(MusicPlayerEvent::Skip).unwrap();
    }
    // Stops whats currently playing and clears queue
    pub fn stop(&self) {
        self.sender.send(MusicPlayerEvent::Stop).unwrap();
    }
}
