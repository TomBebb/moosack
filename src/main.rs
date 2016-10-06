extern crate sdl2;
extern crate sdl2_mixer;
extern crate gtk;
extern crate sqlite;
extern crate hyper;
mod player;
// mod signal;
mod source;
use gtk::prelude::*;
use gtk::{FileFilter, FileChooserDialog,FileChooserAction, Object, Builder, Window, ToolButton};

use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

use player::{Player, PlayEvent};
use source::Src;

const MIME_TYPES: [&'static str; 4] = [
    "audio/mpeg",
    "audio/ogg",
    "audio/vnd.wav",
    "audio/flac"
];

fn get_builder_obj<'a, T>(builder: &'a mut Builder, name: &str) -> T where T: IsA<Object> {
    if let Some(obj) = builder.get_object(name) {
        obj
    } else {
        panic!("failed to load '{}' from glade", name);
    }
}

#[derive(Clone)]
pub struct Ui {
    pub filter: FileFilter,
    pub window: Window,
    pub import: ToolButton,
    pub play: ToolButton,
    pub pause: ToolButton,
    pub stop: ToolButton,
    pub skip: ToolButton
}
impl Ui {
    pub fn new(builder: &mut Builder) -> Ui {
        let filter = FileFilter::new();
        for mime in &MIME_TYPES {
            filter.add_mime_type(mime);
        }
        Ui {
            filter: filter,
            window: get_builder_obj(builder, "window"),
            import: get_builder_obj(builder, "import"),
            play: get_builder_obj(builder, "play"),
            pause: get_builder_obj(builder, "pause"),
            stop: get_builder_obj(builder, "stop"),
            skip: get_builder_obj(builder, "skip"),
        }
    }
    pub fn init(&self, player: Arc<Mutex<Player>>) {
        let filter = self.filter.clone();
        let player2 = player.clone();
        let window = self.window.clone();
        self.import.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(Some("Import music"), Some(&window), FileChooserAction::Open);
            dialog.set_filter(&filter);
            dialog.add_button("Add to Playlist", 0);
            dialog.add_button("Cancel", 1);
            dialog.set_select_multiple(true);
            dialog.show_all();
            let player = player2.clone();
            dialog.connect_response(move |dialog, r| {
                if r == 0 {
                    let mut player = player.lock().unwrap();
                    for uri in dialog.get_uris() {
                        player.queue(Src::Url(&uri));
                    }
                }
                dialog.destroy();
            });
        });
        let player2 = player.clone();
        self.play.connect_clicked(move |_|
            player2.lock().unwrap().play());

        let player2 = player.clone();
        self.pause.connect_clicked(move |_|
            player2.lock().unwrap().pause());

        self.window.drag_dest_add_uri_targets();

        let player2 = player.clone();
        self.window.connect_drag_data_received(move |_, _, _, _, data, _, _| {
            let mut player = player2.lock().unwrap();
            for uri in data.get_uris() {
                player.queue(Src::Url(&uri));
            }
        });
        let player2 = player.clone();
        self.skip.connect_clicked(move |_| {
            player2.lock().unwrap().skip();
        });

        let player2 = player.clone();
        self.stop.connect_clicked(move |_|
            player2.lock().unwrap().stop());
        
        let player2 = player.clone();
        self.window.connect_delete_event(move |_, _| {
            let mut player = player2.lock().unwrap();
            player.stop();
            MAIN_RUNNING.with(|s| s.set(false));
            Inhibit(false)
        });
        self.window.show_all();
    }
    pub fn update(&self, playing: bool) {
        self.play.set_sensitive(!playing);
        self.pause.set_sensitive(playing);
        self.stop.set_sensitive(playing);
        self.skip.set_sensitive(true);
    }
    pub fn update_title(&self, left: usize) {
        let title = format!("Moosack [{}]", left);
        self.window.set_title(&title);
    }
}

thread_local!{
    pub static MAIN_RUNNING: Cell<bool> = Cell::new(false);
}

pub struct App {
    player: Arc<Mutex<Player>>,
    ui: Ui
}
impl App {
    pub fn new(builder: &mut Builder) -> App {
        App {
            player: Arc::new(Mutex::new(Player::new())),
            ui: Ui::new(builder)
        }
    }
    pub fn init(&self) {
        self.ui.init(self.player.clone());
    }
    pub fn main(&self) {
        let ui = Arc::new(self.ui.clone());
        let player = self.player.clone();
        MAIN_RUNNING.with(|s| s.set(true));
        while MAIN_RUNNING.with(Cell::get) {
            gtk::main_iteration();
            if let Ok(player) = player.try_lock() {
                let recv = player.get_receiver();
                while let Ok((event, source)) = recv.try_recv() {
                    println!("{:?}, {:?}", event, source);
                    match event {
                        PlayEvent::Play | PlayEvent::Resume => ui.update(true),
                        PlayEvent::Stop | PlayEvent::Pause => ui.update(false),
                    }
                    ui.update_title(player.get_queue_left());
                }
            }
        }
        self.ui.window.destroy();
    }
}

fn main() {
	if gtk::init().is_err() {
		println!("Failed to initialise GTK");
		return;
	};
    let glade_src = include_str!("window.glade");
    let mut builder = Builder::new_from_string(glade_src);
    let app = App::new(&mut builder);
    app.init();
    app.main();
}