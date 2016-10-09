extern crate gtk;
extern crate gdk;
extern crate moosack;
extern crate vlc;

use moosack::{Config, EventType, Player, Src, MUSIC_EXT};

use gtk::prelude::*;
use gtk::{Builder, FileFilter, FileChooserDialog, FileChooserAction, Object, ToolButton, Window};
use gdk::EventKey;

use vlc::Meta;

use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

fn get_builder_obj<'a, T>(builder: &'a mut Builder, name: &str) -> T
    where T: IsA<Object>
{
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
    pub skip: ToolButton,
}
impl Ui {
    pub fn new(builder: &mut Builder) -> Ui {
        let filter = FileFilter::new();
        for ext in &MUSIC_EXT {
            filter.add_pattern(&format!("*.{}", ext));
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
        self.window.set_icon_name(Some("applications-multimedia"));
        let player2 = player.clone();
        let window = self.window.clone();
        self.window.connect_event(move |_, e| {
            if let Ok(e) = e.clone().downcast::<EventKey>() {
                match (e.get_event_type(), e.get_keyval()) {
                    (gdk::EventType::KeyPress, 32) => {
                        let mut player = player2.lock().unwrap();
                        player.toggle_playing();
                    }
                    (gdk::EventType::KeyPress, 65363) => {
                        let mut player = player2.lock().unwrap();
                        player.skip();
                    }
                    (gdk::EventType::KeyPress, key) => println!("Key: {:?}", key),
                    _ => (),
                };
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
        let filter = self.filter.clone();
        let player2 = player.clone();
        self.import.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(Some("Import music"),
                                                Some(&window),
                                                FileChooserAction::Open);
            dialog.set_filter(&filter);
            dialog.add_button("Play", 0);
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
        self.play.connect_clicked(move |_| player2.lock().unwrap().play());

        let player2 = player.clone();
        self.pause.connect_clicked(move |_| player2.lock().unwrap().pause());

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
        self.stop.connect_clicked(move |_| player2.lock().unwrap().stop());

        let player2 = player.clone();
        self.window.connect_delete_event(move |_, _| {
            let mut player = player2.lock().unwrap();
            player.stop();
            RUNNING.store(false, Ordering::Relaxed);
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
    pub fn update_title(&self, title: &str) {
        let title = format!("Moosack - {}", title);
        self.window.set_title(&title);
    }
}

static RUNNING: AtomicBool = ATOMIC_BOOL_INIT;

pub struct App {
    player: Arc<Mutex<Player>>,
    ui: Ui,
}
impl App {
    pub fn new(builder: &mut Builder) -> App {
        App {
            player: Arc::new(Mutex::new(Player::new())),
            ui: Ui::new(builder),
        }
    }
    pub fn init(&self) {
        self.ui.init(self.player.clone());
    }
    pub fn main(&self) {
        let config = Config::new();
        {
            let mut p = self.player.lock().unwrap();
            let files = config.scan();
            for file in &files {
                p.queue(Src::File(&file));
            }
        };
        let p = self.player.clone();
        let ui = self.ui.clone();
        RUNNING.store(true, Ordering::Relaxed);
        while RUNNING.load(Ordering::Relaxed) {
            let mut p = p.lock().unwrap();
            while let Some(e) = p.poll() {
                match e.ty {
                    EventType::Play | EventType::Resume => ui.update(true),
                    EventType::Stop | EventType::Pause => ui.update(false),
                }
                let media = p.loader.get_media(Src::from(&e.src)).unwrap();
                if let Some(title) = media.get_meta(Meta::Title) {
                    ui.update_title(&title);
                }
            }
            mem::drop(p);
            gtk::main_iteration();
        }
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
