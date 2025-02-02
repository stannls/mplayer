use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    ops::Deref,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use rodio::{Decoder, OutputStream, Sink};

use super::{Album, Song};

// This struct represents all possible interactions with the music player
pub enum MusicPlayerEvent {
    Stop,
    Play(Box<(Decoder<BufReader<File>>, SongInfo)>),
    Skip,
    Pause,
    Volume(f32),
    None,
}

// Represents a Song played by the player
#[derive(Clone)]
pub struct SongInfo {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub length: usize,
    play_start: Option<Instant>,
    paused_at: Option<Instant>,
}

impl SongInfo {
    pub fn new(name: String, artist: String, album: String, length: usize) -> SongInfo {
        SongInfo {
            name,
            artist,
            album,
            length,
            play_start: None,
            paused_at: None,
        }
    }
    pub fn set_start(mut self, start: Instant) -> SongInfo {
        self.play_start = Some(start);
        self
    }
    pub fn played_time(&self) -> Option<usize> {
        if self.play_start.is_some() {
            if self.paused_at.is_none() {
                Some((Instant::now() - self.play_start.unwrap()).as_secs() as usize)
            } else {
                Some((self.paused_at.unwrap() - self.play_start.unwrap()).as_secs() as usize)
            }
        } else {
            None
        }
    }
    pub fn set_paused(mut self) -> SongInfo {
        self.paused_at = Some(Instant::now());
        self
    }
    pub fn unpause(mut self) -> SongInfo {
        if self.paused_at.is_some() {
            self.play_start =
                Some(Instant::now() - (self.paused_at.unwrap() - self.play_start.unwrap()));
            self.paused_at = None;
        }
        self
    }
}

#[derive(Clone)]
pub struct MusicPlayer {
    // These two need to be stored in the struct, because else they will go out of scope and the
    // sink will be unable to play
    sender: Sender<MusicPlayerEvent>,
    current_song: Arc<Mutex<Option<SongInfo>>>,
    queue: Arc<Mutex<VecDeque<SongInfo>>>,
}

impl MusicPlayer {
    pub fn new() -> MusicPlayer {
        let (tx, rx) = mpsc::channel::<MusicPlayerEvent>();
        let played_song = Arc::new(Mutex::new(None));
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        MusicPlayer::start(rx, played_song.to_owned(), queue.to_owned());
        MusicPlayer {
            sender: tx,
            current_song: played_song,
            queue,
        }
    }
    fn start(
        rx: Receiver<MusicPlayerEvent>,
        current_song: Arc<Mutex<Option<SongInfo>>>,
        queue: Arc<Mutex<VecDeque<SongInfo>>>,
    ) {
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let mut internal_queue = VecDeque::new();
            loop {
                match rx.try_recv().unwrap_or(MusicPlayerEvent::None) {
                    MusicPlayerEvent::Play(source) => {
                        sink.play();
                        if sink.empty() {
                            sink.append(source.0);
                            let mut guard = current_song.lock().unwrap();
                            *guard = Some(source.1.set_start(Instant::now()));
                        } else {
                            queue.lock().unwrap().push_back(source.1.to_owned());
                            internal_queue.push_back(source);
                        }
                    }
                    MusicPlayerEvent::Stop => {
                        sink.stop();
                        internal_queue.clear();
                        let mut guard = current_song.lock().unwrap();
                        *guard = None;
                    }
                    MusicPlayerEvent::Skip => {
                        if !sink.empty() && !internal_queue.is_empty() {
                            sink.stop();
                            let song = internal_queue.pop_front().unwrap();
                            sink.append(song.0);
                            let mut guard = current_song.lock().unwrap();
                            *guard = Some(song.1.set_start(Instant::now()));
                            queue.lock().unwrap().pop_front();
                        }
                    }
                    MusicPlayerEvent::Pause => {
                        if sink.is_paused() && !sink.empty() {
                            let mut guard = current_song.lock().unwrap();
                            *guard = Some(guard.to_owned().unwrap().unpause());
                            sink.play()
                        } else if !sink.empty() {
                            let mut guard = current_song.lock().unwrap();
                            *guard = Some(guard.to_owned().unwrap().set_paused());
                            sink.pause();
                        }
                    }
                    MusicPlayerEvent::Volume(v) => {
                        if sink.volume() + v > 1.0 {
                            sink.set_volume(1.0);
                        } else if sink.volume() + v < 0.0 {
                            sink.set_volume(0.0);
                        } else {
                            sink.set_volume(sink.volume() + v);
                        }
                    }
                    MusicPlayerEvent::None => {}
                }
                // Event for playing a new song after the last is finished
                if !internal_queue.is_empty() && sink.empty() {
                    let song = internal_queue.pop_front().unwrap();
                    sink.append(song.0);
                    let mut guard = current_song.lock().unwrap();
                    *guard = Some(song.1.set_start(Instant::now()));
                    queue.lock().unwrap().pop_front();
                } else if sink.empty() && current_song.lock().unwrap().is_some() {
                    let mut guard = current_song.lock().unwrap();
                    *guard = None;
                }
                thread::sleep(Duration::from_millis(50))
            }
        });
    }
    // Emptys queue, plays song
    pub fn play_song(&self, song: Box<dyn Song>, stop_playback: bool) {
        let file = BufReader::new(File::open(song.get_filepath().unwrap()).unwrap());
        let source = Decoder::new(file).unwrap();
        if stop_playback {
            self.stop();
        }
        self.sender
            .send(MusicPlayerEvent::Play(Box::new((
                source,
                SongInfo::new(
                    song.get_title(),
                    song.get_artist_name(),
                    song.get_album_name(),
                    song.get_length_secs().unwrap(),
                ),
            ))))
            .unwrap();
    }
    // Emptys queue, enqueues album
    pub fn play_album(&self, album: Box<dyn Album>, stop_playback: bool) {
        if stop_playback {
            self.stop();
        }
        for song in album.get_songs() {
            let file = BufReader::new(File::open(song.get_filepath().unwrap()).unwrap());
            let source = Decoder::new(file).unwrap();
            self.sender
                .send(MusicPlayerEvent::Play(Box::new((
                    source,
                    SongInfo::new(
                        song.get_title(),
                        song.get_artist_name(),
                        song.get_album_name(),
                        song.get_length_secs().unwrap(),
                    ),
                ))))
                .unwrap();
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
    pub fn get_song_info(&self) -> Option<SongInfo> {
        self.current_song.lock().unwrap().deref().to_owned()
    }
    pub fn change_volume(&self, modifier: f32) {
        self.sender
            .send(MusicPlayerEvent::Volume(modifier))
            .unwrap();
    }
    pub fn get_queue(&self) -> VecDeque<SongInfo> {
        self.queue.lock().unwrap().deref().to_owned()
    }
}
