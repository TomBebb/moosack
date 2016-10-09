//! Moosack is a music-playing library.
extern crate hyper;
extern crate rustc_serialize;
extern crate sqlite;
extern crate toml;
extern crate vlc;

use std::fs;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// File extensions recognised as music player backend (VLC).
pub const MUSIC_EXT: [&'static str; 18] = ["aac", "ac3", "dta", "flac", "m4a", "m4p", "mka",
                                           "mod", "mp1", "mp2", "mp3", "ogg", "oma", "pls", "raw",
                                           "spx", "wav", "wma"];

/// File extensions recognised as music player backend (VLC).
pub const PLAYLIST_EXT: [&'static str; 4] = ["b4s", "cue", "m3u", "xspf"];

pub fn is_music(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        let ext: String = ext.to_lowercase();
        MUSIC_EXT.contains(&&*ext)
    } else {
        false
    }
}
pub fn is_playlist(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        let ext: String = ext.to_lowercase();
        PLAYLIST_EXT.contains(&&*ext)
    } else {
        false
    }
}

fn scan_dir(dir: &Path, items: &mut Vec<PathBuf>) {
    for file in fs::read_dir(dir).unwrap() {
        let file = file.unwrap().path();
        if file.is_dir() {
            scan_dir(&file, items);
        } else if is_music(&file) {
            items.push(file);
        }
    }
}

mod config;
mod source;
mod signal;
mod player;

pub use config::Config;
pub use player::{Player, EventType, Event};
pub use source::{Source, Src, Loader};
pub use signal::Signal;
