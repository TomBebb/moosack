use std::collections::VecDeque;
use source::{Source, Src, Loader};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use vlc::{self, Media, MediaPlayer, State};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum EventType {
    Play,
    Pause,
    Resume,
    Stop,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Event {
    pub ty: EventType,
    pub src: Source,
}
impl Event {
    pub fn new(ty: EventType, src: Source) -> Event {
        Event { ty: ty, src: src }
    }
}

pub struct LoadedMusic {
    pub src: Source,
    pub media: Media,
}
impl LoadedMusic {
    pub fn new(loader: &mut Loader, source: Src) -> Result<LoadedMusic, String> {
        loader.load(source).map(|media| {
            LoadedMusic {
                src: source.into(),
                media: media,
            }
        })
    }
}

pub struct Player {
    current: Option<LoadedMusic>,
    queue: VecDeque<Source>,
    player: MediaPlayer,
    pub loader: Loader,
    pub events: Vec<Event>,
    ended: Arc<AtomicBool>,
}
impl Player {
    pub fn get_queue_left(&self) -> usize {
        self.queue.len()
    }
    pub fn new() -> Player {
        let loader = Loader::new();
        let player = MediaPlayer::new(&loader.instance).unwrap();
        let events = Vec::with_capacity(16);
        let ended = Arc::new(AtomicBool::new(false));
        let ended2 = ended.clone();
        {
            let em = player.event_manager();
            let _ = em.attach(vlc::EventType::MediaPlayerPlaying, move |_, _| {
                println!("playing");
            });
            let _ = em.attach(vlc::EventType::MediaPlayerPaused, move |_, _| {
                println!("paused");
            });
            let _ = em.attach(vlc::EventType::MediaPlayerEndReached, move |_, _| {
                ended2.store(true, Ordering::Relaxed);
                println!("got to end");
            });
        }
        Player {
            events: events,
            current: None,
            player: player,
            ended: ended,
            queue: VecDeque::with_capacity(16),
            loader: loader,
        }
    }
    fn start_playing(&mut self, source: Src) {
        let m = LoadedMusic::new(&mut self.loader, source).unwrap();
        self.player.set_media(&m.media);
        self.player.play().unwrap();
        self.current = Some(m);
        self.events.push(Event::new(EventType::Play, source.into()));
    }
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }
    pub fn queue(&mut self, source: Src) {
        if !self.current.is_some() {
            self.start_playing(source);
        } else {
            self.queue.push_back(source.into());
        }
    }
    pub fn toggle_playing(&mut self) {
        if !self.is_playing() {
            self.play();
        } else {
            self.pause();
        }
    }
    pub fn pause(&mut self) {
        if let Some(ref m) = self.current {
            self.player.pause();
            self.events.push(Event::new(EventType::Pause, m.src.clone()));
        }
    }
    pub fn play(&mut self) {
        let s = self.player.state();
        match s {
            State::Stopped | State::Paused | State::NothingSpecial => {
                if let Some(ref m) = self.current {
                    self.player.play().unwrap();
                    let ty = if s == State::Stopped {
                        EventType::Play
                    } else {
                        EventType::Resume
                    };
                    self.events.push(Event::new(ty, m.src.clone()));
                } else {
                    self.skip();
                }
            }
            _ => (),
        }
    }
    pub fn play_now(&mut self, new: Src) {
        if let Some(ref m) = self.current {
            self.events.push(Event::new(EventType::Stop, m.src.clone()));
            self.player.stop();
        }
        self.start_playing(new);
    }
    pub fn skip(&mut self) {
        self.current = None;
        if let Some(ref next) = self.queue.pop_front() {
            self.play_now(next.into());
        }
    }
    pub fn stop(&mut self) {
        if let Some(ref m) = self.current {
            self.events.push(Event::new(EventType::Stop, m.src.clone()));
        }
        self.queue.clear();
        self.current = None;
    }
    pub fn exit(self) {
        self.player.stop();
    }
    pub fn get_position(&self) -> Option<f32> {
        self.player.get_position()
    }
    pub fn poll(&mut self) -> Option<Event> {
        if self.ended.load(Ordering::Relaxed) {
            self.ended.store(false, Ordering::Relaxed);
            self.skip();
        }
        self.events.pop()
    }
}
