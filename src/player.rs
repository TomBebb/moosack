use std::collections::VecDeque;
use source::{Source, Src, Loader};

use sdl2_mixer::{self, Music};

use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PlayEvent {
	Play,
	Pause,
	Resume,
	Stop
}

pub struct Player {
	offset: u64,
	current: Option<Music>,
	current_source: Option<Source>,
	queue: VecDeque<Source>,
	loader: Loader,
	receiver: Receiver<(PlayEvent, Source)>,
	sender: Sender<(PlayEvent, Source)>
}
impl Player {
	pub fn get_queue_left(&self) -> usize {
		self.queue.len()
	}
	pub fn new() -> Player {
	    sdl2_mixer::open_audio(
	        sdl2_mixer::DEFAULT_FREQUENCY,
	        sdl2_mixer::DEFAULT_FORMAT,
	        sdl2_mixer::DEFAULT_CHANNELS,
	        1024
	    ).unwrap();
	    let (tx, rx) = channel();
	    sdl2_mixer::allocate_channels(sdl2_mixer::DEFAULT_CHANNELS);
		Player {
			sender: tx,
			receiver: rx,
			offset: 0,
			current: None,
			current_source: None,
			queue: VecDeque::with_capacity(16),
			loader: Loader::new()
		}
	}
	pub fn is_playing(&self) -> bool {
		Music::is_playing()
	}
	pub fn queue(&mut self, source: Src) {
		if !self.current.is_some() {
			self.current = Some(self.loader.load(source).unwrap());
			self.current_source = Some(source.into());
			self.current.as_mut().unwrap().play(1).unwrap();
			self.sender.send((PlayEvent::Play, source.into())).unwrap();
		} else {
			self.queue.push_back(source.into());
		}
	}
	pub fn toggle_playing(&mut self) {
		if Music::is_paused() {
			self.play();
		} else {
			self.pause();
		}
	}
	pub fn pause(&mut self) {
		if let Some(ref source) = self.current_source {
			Music::pause();
			self.sender.send((PlayEvent::Pause, source.clone())).unwrap();
		}
	}
	pub fn play(&mut self) {
		if Music::is_paused() {
			Music::resume();
			if let Some(ref source) = self.current_source {
				self.sender.send((PlayEvent::Resume, source.clone())).unwrap();
			}
			//Music::hook_finished(music_end);
		} else if !Music::is_playing() {
			if let Some(ref current) = self.current {
				current.play(1).unwrap();
				self.sender.send((PlayEvent::Play, self.current_source.clone().unwrap())).unwrap();
			} else {
				self.skip();
			}
		}
	}
	pub fn play_now(&mut self, new: Src) {
		if Music::is_playing() {
			if let Some(ref source) = self.current_source {
				self.sender.send((PlayEvent::Stop, source.clone())).unwrap();
			}
			Music::halt();
		}
		self.current_source = Some(new.into());
		self.current = Some(self.loader.load(new).unwrap());
		self.current.as_mut().unwrap().play(1).unwrap();
		self.sender.send((PlayEvent::Play, new.into())).unwrap();
		self.offset = 0;
	}
	pub fn skip(&mut self) {
		self.current = None;
		if let Some(ref next) = self.queue.pop_front() {
			self.play_now(next.into());
		}
	}
	pub fn stop(&mut self) {
		if Music::is_playing() {
			Music::halt();
		}
		if let Some(ref source) = self.current_source {
			self.sender.send((PlayEvent::Stop, source.clone())).unwrap();
		}
		self.queue.clear();
		self.current_source = None;
		self.current = None;
	}
	pub fn exit(&mut self) {
		Music::halt();
	}
	pub fn get_receiver(&self) -> &Receiver<(PlayEvent, Source)> {
		&self.receiver
	}
}