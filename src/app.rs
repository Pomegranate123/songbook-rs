use crate::{conf::Config, song::Song, util::StatefulList};
use std::fs;
use tui::widgets::ListItem;

pub struct App<'a> {
    pub items: StatefulList<ListItem<'a>>,
    pub search: Vec<String>,
    pub song: Option<Song<'a>>,
    pub selected_index: Option<usize>,
    pub config: Config,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(
                App::search_songs()
                    .iter()
                    .map(|s| ListItem::new(String::from(s)))
                    .collect(),
            ),
            search: App::search_songs(),
            song: None,
            selected_index: None,
            config: Config::default(),
        }
    }

    fn search_songs() -> Vec<String> {
        let mut paths: Vec<_> = fs::read_dir("/home/pomegranate/Dropbox/Songbook/NL Selectie")
            .unwrap()
            .map(|dir| dir.unwrap())
            .collect();
        paths.sort_by_key(|dir| dir.path());
        paths
            .iter()
            .filter_map(|f| match f.file_name().into_string() {
                Ok(file) => Some(file.trim_end_matches(".txt").to_string()),
                Err(_) => None,
            })
            .collect()
    }

    pub fn load_song(&mut self, selected: Option<usize>) {
        if self.selected_index != selected {
            if let Some(index) = selected {
                let songstring = &self.search[index];
                let path = String::from("/home/pomegranate/Dropbox/Songbook/NL Selectie/")
                    + &songstring[..]
                    + ".txt";
                let file = fs::read_to_string(path).unwrap();
                self.song = Some(Song::new(file, &self.config.theme));
            }
        }
    }
}
