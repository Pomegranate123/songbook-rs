use crate::{conf::Config, song::Song, util::StatefulList};
use std::{collections::HashMap, fs};

pub struct App<'a> {
    pub songlist: StatefulList<String>,
    pub songs: HashMap<String, String>,
    pub song: Option<Song<'a>>,
    pub config: Config,
    pub input: String,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let config = Config::default();
        let songs = App::list_songs(&config);
        let mut items: Vec<String> = songs.iter().map(|s| String::from(s.0)).collect();
        items.sort();
        App {
            songlist: StatefulList::with_items(items),
            songs,
            song: None,
            config: Config::default(),
            input: String::default(),
        }
    }

    pub fn search_songs(&self, query: &str) -> Vec<String> {
        let mut results: Vec<String>;
        results = self
            .songs
            .iter()
            .filter_map(|(f, s)| {
                if s.to_lowercase().contains(&query.to_lowercase())
                    | f.to_lowercase().contains(&query.to_lowercase())
                {
                    Some(f.clone())
                } else {
                    None
                }
            })
            .collect();
        results.sort();
        results
    }

    fn list_songs(config: &Config) -> HashMap<String, String> {
        let mut paths: Vec<_> = fs::read_dir(&config.path)
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
        paths.sort_by_key(|dir| dir.path());
        paths
            .iter()
            .filter_map(|f| {
                let filename = f.file_name();
                let filename = filename.to_str().unwrap();
                if filename.ends_with(".txt") {
                    Some((
                        String::from(filename.trim_end_matches(".txt")),
                        fs::read_to_string(f.path()).unwrap(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn load_song(&mut self, selected: Option<usize>) {
        if let Some(index) = selected {
            match self.songlist.items.get(index) {
                Some(song) => {
                    self.song = Some(Song::new(self.songs[song].clone(), &self.config.theme))
                }
                None => self.songlist.select(None),
            }
        }
    }
}
