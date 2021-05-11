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
        let mut items: Vec<String> = App::list_songs()
            .iter()
            .map(|s| String::from(s.0))
            .collect();
        items.sort();
        App {
            songlist: StatefulList::with_items(items),
            songs: App::list_songs(),
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

    fn list_songs() -> HashMap<String, String> {
        let mut paths: Vec<_> = fs::read_dir("/home/pomegranate/Dropbox/Songbook/NL Selectie")
            .unwrap()
            .map(|dir| dir.unwrap())
            .collect();
        paths.sort_by_key(|dir| dir.path());
        paths
            .iter()
            .map(|f| {
                (
                    String::from(f.file_name().to_str().unwrap().trim_end_matches(".txt")),
                    fs::read_to_string(f.path()).unwrap(),
                )
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
