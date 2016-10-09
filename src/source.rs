use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use vlc::{Instance, Media};

/// A source loader, capable of caching audio sources.
pub struct Loader {
    pub instance: Instance,
    cache: HashMap<Source, Media>,
}
impl Loader {
    /// Create a new loader, with an empty cache.
    pub fn new() -> Loader {
        Loader {
            instance: Instance::new().unwrap(),
            cache: HashMap::with_capacity(8),
        }
    }
    pub fn get_media(&mut self, source: Src) -> Result<&Media, String> {
        let src = Source::from(source);
        if !self.cache.contains_key(&src) {
            let media = try!(self.load(source));
            self.cache.insert(src.clone(), media);
        }
        Ok(&self.cache[&src])
    }
    /// Load music from the source `source`.
    pub fn load(&mut self, source: Src) -> Result<Media, String> {
        match source {
            Src::File(path) => {
                match Media::new_path(&self.instance, path.to_str().unwrap()) {
                    Some(media) => Ok(media),
                    None => Err(format!("Failed to open file {:?}", path)),
                }
            }
            Src::Url(url) => {
                match Media::new_location(&self.instance, url) {
                    Some(media) => Ok(media),
                    None => Err(format!("Failed to stream from URL {:?}", url)),
                }
            }
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Src<'a> {
    File(&'a Path),
    Url(&'a str),
}
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum Source {
    File(Rc<PathBuf>),
    Url(Rc<String>),
}
impl<'a> From<&'a Source> for Src<'a> {
    fn from(s: &'a Source) -> Src<'a> {
        match *s {
            Source::File(ref p) => Src::File(p),
            Source::Url(ref u) => Src::Url(u),
        }
    }
}
impl<'a> From<Src<'a>> for Source {
    fn from(s: Src<'a>) -> Source {
        match s {
            Src::File(p) => Source::File(Rc::new(p.into())),
            Src::Url(u) => Source::Url(Rc::new(u.into())),
        }
    }
}
