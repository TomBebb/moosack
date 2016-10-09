use std::default::Default;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use toml;

const DEFAULT_MUSIC_DIRS: [&'static str; 3] = ["Music", "music", "My Music"];
const CONFIG_PATH: &'static str = ".moosack";
#[derive(RustcEncodable, RustcDecodable)]
pub struct Config {
    pub libraries: Vec<PathBuf>,
}
impl Config {
    fn path() -> PathBuf {
        let mut path = env::home_dir().unwrap();
        path.push(CONFIG_PATH);
        path
    }
    pub fn new() -> Config {
        let path = Config::path();
        if path.exists() {
            return Config::new_from_file(&path);
        } else {
            return Config::default();
        }
    }
    pub fn save(&self) {
        let mut file = File::open(Config::path()).unwrap();
        file.write(toml::encode_str(self).as_bytes()).unwrap();
    }
    pub fn new_from_file(path: &Path) -> Config {
        let mut file = File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        toml::decode_str(&buf).unwrap_or_else(Config::default)
    }
    pub fn scan(&self) -> Vec<PathBuf> {
        let mut items = Vec::with_capacity(8);
        for dir in &self.libraries {
            ::scan_dir(&dir, &mut items);
        }
        items
    }
}
impl Default for Config {
    fn default() -> Config {
        let libraries = env::home_dir()
            .map(|mut path| {
                for dir in &DEFAULT_MUSIC_DIRS {
                    path.push(dir);
                    if path.exists() {
                        break;
                    }
                    path.pop();
                }
                vec![path]
            })
            .unwrap_or_else(Vec::new);
        Config { libraries: libraries }
    }
}
