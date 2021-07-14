use crate::{
    conf::Config,
    parser::{Playlist, Song},
};
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use tui::widgets::ListState;

#[derive(Default)]
pub struct App {
    files: HashMap<FileType, String>,
    pub file_nav: FileNavigator,
    pub search_nav: FileNavigator,
    pub config: Config,
    pub song: Option<Song>,
    pub searching: bool,
    pub input: String,
    pub path: Vec<String>,
    pub transposing: bool,
}

impl<'a> App {
    pub fn new(config: Config) -> Self {
        let files = App::create_filemap(&config.path);
        let mut all_files: Vec<FileType> = files.keys().cloned().collect();
        all_files.sort_by_key(FileType::name);
        App {
            file_nav: FileNavigator::from_path(&config.path),
            search_nav: FileNavigator(vec![Folder {
                name: String::from("Search"),
                files: all_files,
                state: ListState::default(),
            }]),
            files,
            config,
            ..Default::default()
        }
    }

    pub fn load_selected(&mut self) {
        let file = self.get_nav().selected().cloned();
        if let Some(file) = file {
            match file {
                FileType::Folder(path) => self.get_nav_mut().open_path(&path),
                FileType::Song(_) => {
                    self.song = Some(Song::from(
                        self.files
                            .get(&file)
                            .unwrap_or(&String::from("{title:Song not found}"))
                            .clone(),
                    ))
                }
                FileType::Playlist(_) => {
                    let playlist = Playlist::from(self.files.get(&file).unwrap());
                    self.get_nav_mut().open_playlist(playlist)
                }
            }
        }
    }

    pub fn load_selected_song(&mut self) {
        if let Some(FileType::Song(_)) = self.get_nav().selected() {
            self.load_selected()
        }
    }

    pub fn search(&mut self) {
        let input = &self.input.to_lowercase();
        let mut results: Vec<FileType> = self
            .files
            .iter()
            .filter_map(|(k, v)| {
                if k.name().to_lowercase().contains(input) | v.to_lowercase().contains(input) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        results.sort_by_key(FileType::name);
        self.get_nav_mut().0 = vec![Folder {
            name: String::from("Search"),
            files: results,
            state: ListState::default(),
        }];
    }

    fn create_filemap(path: &Path) -> HashMap<FileType, String> {
        App::get_direntries(path)
            .iter()
            .filter_map(|file| {
                let path = file.path();
                if path.is_dir() {
                    Some((FileType::Folder(path), String::new()))
                } else {
                    let extension = path.extension().unwrap().to_str().unwrap();
                    if extension == "txt" {
                        let filestring = fs::read_to_string(file.path()).unwrap();
                        Some((FileType::Song(Song::get_name(&filestring)), filestring))
                    } else if extension == "lst" {
                        let filestring = fs::read_to_string(file.path()).unwrap();
                        Some((
                            FileType::Playlist(Playlist::get_name(&filestring)),
                            filestring,
                        ))
                    } else {
                        None
                    }
                }
            })
            .collect()
    }

    // Gets all DirEntry's that are not a folder
    fn get_direntries(path: &Path) -> Vec<DirEntry> {
        fs::read_dir(path)
            .unwrap()
            .flat_map(|dir| {
                let dir = dir.unwrap();
                let path = dir.path();
                if path.is_dir() {
                    let mut dirs = App::get_direntries(&path);
                    dirs.push(dir);
                    dirs
                } else {
                    vec![dir]
                }
            })
            .collect()
    }

    pub fn get_nav(&self) -> &FileNavigator {
        if self.searching {
            &self.search_nav
        } else {
            &self.file_nav
        }
    }

    pub fn get_nav_mut(&mut self) -> &mut FileNavigator {
        if self.searching {
            &mut self.search_nav
        } else {
            &mut self.file_nav
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FileType {
    Folder(PathBuf),
    Song(String),
    Playlist(String),
}

impl FileType {
    pub fn from_dir_entry(entry: DirEntry) -> Result<FileType, &'static str> {
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            Ok(FileType::Folder(path))
        } else if name.ends_with(".txt") {
            Ok(FileType::Song(Song::get_name(
                &fs::read_to_string(path).unwrap(),
            )))
        } else if name.ends_with(".lst") {
            Ok(FileType::Playlist(Playlist::get_name(
                &fs::read_to_string(path).unwrap(),
            )))
        } else {
            Err("Unable to parse DirEntry to File")
        }
    }

    pub fn name(&self) -> String {
        match self {
            FileType::Folder(path) => path.file_name().unwrap().to_str().unwrap().to_owned(),
            FileType::Song(name) => name.to_owned(),
            FileType::Playlist(name) => name.to_owned(),
        }
    }
}

#[derive(Default)]
pub struct Folder {
    pub name: String,
    pub state: ListState,
    pub files: Vec<FileType>,
}

impl Folder {
    fn from_path(path: &Path) -> Folder {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let mut files: Vec<FileType> = fs::read_dir(path)
            .unwrap()
            .filter_map(|dir| FileType::from_dir_entry(dir.unwrap()).ok())
            .collect();
        files.sort_by_key(FileType::name);
        Folder {
            name,
            files,
            ..Default::default()
        }
    }

    fn from_playlist(playlist: Playlist) -> Folder {
        Folder {
            name: playlist.title,
            files: playlist.songs,
            ..Default::default()
        }
    }

    fn forward(&mut self, amount: usize) {
        self.state.select(Some(match self.state.selected() {
            Some(mut i) => {
                i += amount;
                if i > self.files.len() - 1 {
                    i -= self.files.len();
                }
                i
            }
            None => 0,
        }))
    }

    fn back(&mut self, amount: usize) {
        self.state.select(Some(match self.state.selected() {
            Some(mut i) => {
                if amount > i {
                    i += self.files.len()
                }
                i -= amount;
                i
            }
            None => 0,
        }))
    }

    fn selected(&self) -> Option<&FileType> {
        if let Some(index) = self.state.selected() {
            return Some(&self.files[index]);
        }
        None
    }
}

#[derive(Default)]
pub struct FileNavigator(Vec<Folder>);

impl FileNavigator {
    fn from_path(path: &Path) -> FileNavigator {
        FileNavigator(vec![Folder::from_path(path)])
    }

    fn open_playlist(&mut self, playlist: Playlist) {
        self.0.push(Folder::from_playlist(playlist));
    }

    fn open_path(&mut self, path: &Path) {
        self.0.push(Folder::from_path(path))
    }

    pub fn path_back(&mut self) {
        if self.0.len() > 1 {
            self.0.pop();
        }
    }

    pub fn current(&self) -> &Folder {
        self.0.last().unwrap()
    }

    pub fn current_mut(&mut self) -> &mut Folder {
        self.0.iter_mut().last().unwrap()
    }

    pub fn forward(&mut self, amount: usize) {
        self.current_mut().forward(amount)
    }

    pub fn back(&mut self, amount: usize) {
        self.current_mut().back(amount)
    }

    fn selected(&self) -> Option<&FileType> {
        self.current().selected()
    }
}
