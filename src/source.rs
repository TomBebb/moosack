use hyper::{Client, Url};
use std::io::{Read, Write};
use std::fs::File;
use std::rc::Rc;
use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use sdl2_mixer::{self, Music, INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG};
use sdl2;

const FILE_URL: &'static str = "file://";

pub struct Loader {
	urls: HashMap<String, PathBuf>
}
impl Loader {
	pub fn new() -> Loader {
	    let sdl = sdl2::init().unwrap();
	    sdl.audio().unwrap();
	    sdl.timer().unwrap();
	    sdl2_mixer::init(INIT_MP3 | INIT_FLAC | INIT_MOD | INIT_FLUIDSYNTH | INIT_MODPLUG | INIT_OGG).unwrap();
	    Loader {
	    	urls: HashMap::with_capacity(16)
	    }
	}
	pub fn load(&mut self, source: Src) -> Result<Music, String> {
		match source {
			Src::File(path) => {
				Music::from_file(&path)
			},
			Src::Url(url) if url.starts_with(FILE_URL) => {
				let url = Url::parse(url).unwrap();
				let file = Url::to_file_path(&url).unwrap();
				Music::from_file(&file)
			},
			Src::Url(url) if self.urls.contains_key(url) =>
				Music::from_file(&self.urls[url]),
			Src::Url(url) => {
				let mut path = env::temp_dir();
				path.push(self.urls.len().to_string());
				let mut file = File::create(&path).unwrap();
				self.urls.insert(url.to_owned(), path.clone());
				let mut response = Client::new().get(url).send().unwrap();
				let mut buf = Vec::with_capacity(32);
				response.read_to_end(&mut buf).unwrap();
				file.write(&buf).unwrap();
				Music::from_file(&path)
			}
		}
	} 
}
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Src<'a> {
	File(&'a Path),
	Url(&'a str)
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Source {
	File(Rc<PathBuf>),
	Url(Rc<String>)
}
impl<'a> From<&'a Source> for Src<'a> {
	fn from(s: &'a Source) -> Src<'a> {
		match *s {
			Source::File(ref p) => Src::File(p),
			Source::Url(ref u) => Src::Url(u)
		}
	}
}
impl<'a> From<Src<'a>> for Source {
	fn from(s: Src<'a>) -> Source {
		match s {
			Src::File(p) => Source::File(Rc::new(p.into())),
			Src::Url(u) => Source::Url(Rc::new(u.into()))
		}
	}
}