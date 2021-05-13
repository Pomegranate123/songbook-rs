use crate::{
    conf::Config,
    parser::{Playlist, Song},
    util::{FolderEntry, StatefulList},
};
use std::{collections::HashMap, fs};

#[derive(PartialEq)]
pub enum AppState {
    Songs,
    Playlists,
    Playlist,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Songs
    }
}

#[derive(Default)]
pub struct App<'a> {
    pub songs: StatefulList,
    pub playlists: StatefulList,
    pub playlist: StatefulList,
    pub config: Config,
    pub state: AppState,
    pub song: Option<Song<'a>>,
    pub searching: bool,
    pub input: String,
    pub path: Vec<String>,
}

impl<'a> App<'a> {
    pub fn new(config: Config) -> Self {
        let playlist_map = App::map_playlists(&config.path);
        let playlists: Vec<_> = playlist_map.iter().map(|s| s.0.clone()).collect();
        App {
            songs: StatefulList::with_items_sorted(
                App::list_songs(&config.path),
                App::map_songs(&config.path),
            ),
            playlists: StatefulList::with_items_sorted(playlists, playlist_map),
            config,
            ..Default::default()
        }
    }

    fn list_songs(path: &str) -> Vec<FolderEntry> {
        let mut files: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|dir| dir.unwrap())
            .collect();
        files.sort_by_key(|dir| dir.path());
        files
            .iter()
            .filter_map(|file| {
                let filename = file.file_name();
                let filename = filename.to_str().unwrap();
                if file.path().is_dir() {
                    Some(FolderEntry::Folder(filename.to_owned()))
                } else if filename.ends_with(".txt") {
                    Some(FolderEntry::File(Song::get_name(
                        &fs::read_to_string(file.path()).unwrap(),
                    )))
                } else {
                    None
                }
            })
            .collect()
    }

    //    fn map_songs(config: &Config) -> HashMap<FolderEntry, String> {
    //        let mut files: Vec<_> = fs::read_dir(&config.path)
    //            .unwrap()
    //            .flat_map(|dir| {
    //                if dir.as_ref().unwrap().path().is_dir() {
    //                    fs::read_dir(dir.unwrap().path())
    //                        .unwrap()
    //                        .map(|dir| dir.unwrap())
    //                        .collect()
    //                } else {
    //                    vec![dir.unwrap()]
    //                }
    //            })
    //            .collect();
    //        files.sort_by_key(|dir| dir.path());
    //        files
    //            .iter()
    //            .filter_map(|file| {
    //                let filename = file.file_name();
    //                let filename = filename.to_str().unwrap();
    //                let filestring = fs::read_to_string(file.path()).unwrap();
    //                if filename.ends_with(".txt") {
    //                    Some((Song::get_name(&filestring), filestring))
    //                } else {
    //                    None
    //                }
    //            })
    //            .collect()
    //    }
    //
    fn map_songs(path: &str) -> HashMap<FolderEntry, String> {
        let mut files: Vec<_> = fs::read_dir(path)
            .unwrap()
            .flat_map(|dir| {
                if dir.as_ref().unwrap().path().is_dir() {
                    fs::read_dir(dir.unwrap().path())
                        .unwrap()
                        .map(|dir| dir.unwrap())
                        .collect()
                } else {
                    vec![dir.unwrap()]
                }
            })
            .collect();
        files.sort_by_key(|dir| dir.path());
        files
            .iter()
            .filter_map(|file| {
                let filename = file.file_name();
                let filename = filename.to_str().unwrap();
                if file.path().is_dir() {
                    Some((
                        FolderEntry::Folder(filename.to_owned()),
                        file.path().to_str().unwrap().to_owned(),
                    ))
                } else if filename.ends_with(".txt") {
                    let filestring = fs::read_to_string(file.path()).unwrap();
                    Some((FolderEntry::File(Song::get_name(&filestring)), filestring))
                } else {
                    None
                }
            })
            .collect()
    }

    fn map_playlists(path: &str) -> HashMap<FolderEntry, String> {
        let mut paths: Vec<_> = fs::read_dir(path)
            .unwrap()
            .filter_map(|dir| {
                if dir.as_ref().unwrap().path().is_dir() {
                    None
                } else {
                    Some(dir.unwrap())
                }
            })
            .collect();
        paths.sort_by_key(|dir| dir.path());
        paths
            .iter()
            .filter_map(|f| {
                let filename = f.file_name();
                let filename = filename.to_str().unwrap();
                let filestring = fs::read_to_string(f.path()).unwrap();
                if filename.ends_with(".lst") {
                    Some((
                        FolderEntry::File(String::from(filename.trim_end_matches(".lst"))),
                        filestring,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn current_path(&self) -> String {
        self.config.path.clone()
            + &self
                .path
                .iter()
                .map(|f| f.to_owned())
                .intersperse_with(|| String::from("/"))
                .collect::<String>()
    }

    pub fn path_forward(&mut self, folder: &str) {
        self.path.push(folder.to_string());
        let path = self.current_path();
        self.songs.items = App::list_songs(&path);
        self.input = String::new()
    }

    pub fn path_back(&mut self) {
        self.path.pop();
        let path = self.current_path();
        self.songs.items = App::list_songs(&path);
        self.input = String::new()
    }

    pub fn load(&mut self) {
        let list = match self.state {
            AppState::Songs => &mut self.songs,
            AppState::Playlists => &mut self.playlists,
            AppState::Playlist => &mut self.playlist,
        };
        if let Some(index) = list.selected() {
            if let Some(key) = list.items.get(index) {
                match self.state {
                    AppState::Songs | AppState::Playlist => match key {
                        FolderEntry::File(_) => match list.item_map.get(key) {
                            Some(value) => {
                                self.song = Some(Song::new(value.clone(), &self.config.theme))
                            }
                            None => {
                                self.song = Some(Song::new(
                                    String::from("{t:Song not found}"),
                                    &self.config.theme,
                                ))
                            }
                        },

                        FolderEntry::Folder(path) => {
                            self.path.push(path.to_string());
                            let path = self.current_path();
                            self.songs.items = App::list_songs(&path)
                        }
                    },
                    AppState::Playlists => {
                        if let Some(playlist) = list.item_map.get(key) {
                            let playlist = Playlist::new(playlist.clone());
                            self.playlist = StatefulList::with_items(
                                playlist.songs,
                                self.songs.item_map.clone(),
                            );
                            self.state = AppState::Playlist;
                        }
                    }
                }
            }
        }
    }
}
