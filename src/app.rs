use crate::{
    conf::Config,
    parser::{Playlist, Song},
    util::{FileType, StatefulList},
};
use lazy_static::lazy_static;
use regex::Regex;
use rust_music_theory::note::PitchClass;
use std::{collections::HashMap, fs};

lazy_static! {
    static ref RE_SONG_TRANSPOSITION: Regex = Regex::new(r" \[([ABCDEFG][b#]?)\]").unwrap();
}

#[derive(Default)]
pub struct App<'a> {
    pub files: StatefulList,
    pub filemap: HashMap<FileType, String>,
    pub config: Config,
    pub song: Option<Song<'a>>,
    pub searching: bool,
    pub input: String,
    pub path: Vec<String>,
    pub extra_column_size: usize,
}

impl<'a> App<'a> {
    pub fn new(config: Config) -> Self {
        let mut files = App::list_files(&config.path);
        files.sort_by_key(|f| f.get());
        App {
            files: StatefulList::with_items(files),
            filemap: App::map_files(&config.path),
            extra_column_size: config.extra_column_size,
            config,
            ..Default::default()
        }
    }

    fn map_files(path: &str) -> HashMap<FileType, String> {
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
                        FileType::Folder(filename.to_owned()),
                        file.path().to_str().unwrap().to_owned(),
                    ))
                } else if filename.ends_with(".txt") {
                    let filestring = fs::read_to_string(file.path()).unwrap();
                    Some((FileType::Song(Song::get_name(&filestring)), filestring))
                } else if filename.ends_with(".lst") {
                    let filestring = fs::read_to_string(file.path()).unwrap();
                    Some((
                        FileType::Playlist(filename.trim_end_matches(".lst").to_owned()),
                        filestring,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    fn list_files(path: &str) -> Vec<FileType> {
        let mut files: Vec<_> = fs::read_dir(path)
            .unwrap_or_else(|_| panic!("Path {} not found.", path))
            .map(|dir| dir.unwrap())
            .collect();
        files.sort_by_key(|dir| dir.path());
        files
            .iter()
            .filter_map(|file| {
                let filename = file.file_name();
                let filename = filename.to_str().unwrap();
                if file.path().is_dir() {
                    Some(FileType::Folder(filename.to_owned()))
                } else if filename.ends_with(".txt") {
                    Some(FileType::Song(Song::get_name(
                        &fs::read_to_string(file.path()).unwrap(),
                    )))
                } else if filename.ends_with(".lst") {
                    Some(FileType::Playlist(String::from(
                        filename.trim_end_matches(".lst"),
                    )))
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
        self.update_path();
    }

    pub fn path_back(&mut self) {
        self.path.pop();
        self.update_path();
    }

    fn update_path(&mut self) {
        let path = self.current_path();
        self.files.items = App::list_files(&path);
        self.input = String::new();
        self.files.select(None);
    }

    pub fn load_selected(&mut self) {
        if let Some(file) = self.files.get_selected_item() {
            match file {
                FileType::Song(songname) => {
                    match self.filemap.get(&file) {
                        Some(value) => {
                            self.song = Some(Song::from(value.clone(), &self.config.theme))
                        }
                        None => {
                            // This is for songs in playlists. If they are transposed, the last
                            // characters will be the new key. This code tries to find a song
                            // without those characters, and if it finds it, it loads it in the new
                            // key.
                            if let Some(key) = RE_SONG_TRANSPOSITION.captures(songname) {
                                let actual_name = RE_SONG_TRANSPOSITION.replace(songname, "");
                                if let Some(value) =
                                    self.filemap.get(&FileType::Song(actual_name.to_string()))
                                {
                                    self.song = Some(Song::in_key(
                                        value.clone(),
                                        &self.config.theme,
                                        PitchClass::from_str(key.get(1).unwrap().as_str()).unwrap(),
                                    ));
                                    return;
                                }
                            }
                            //TODO: Better handling of missing songs?
                            self.song = Some(Song::from(
                                String::from("{t:Song not found}"),
                                &self.config.theme,
                            ))
                        }
                    }
                }

                FileType::Playlist(_) => {
                    if let Some(playlist) = self.filemap.get(file) {
                        let playlist = Playlist::from(playlist.clone());
                        self.files.items = playlist.songs;
                    }
                    self.files.select(None);
                }
                FileType::Folder(path) => self.path_forward(&path.to_string()),
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<FileType> {
        let mut results: Vec<FileType>;
        results = self
            .filemap
            .iter()
            .filter_map(|(k, v)| {
                if k.get().to_lowercase().contains(&query.to_lowercase())
                    | v.to_lowercase().contains(&query.to_lowercase())
                {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        results.sort_by_key(|f| f.get());
        results
    }
}
