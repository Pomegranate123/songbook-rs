#![allow(dead_code)]

pub mod event;

use std::collections::HashMap;
use tui::widgets::ListState;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum FolderEntry {
    Folder(String),
    File(String),
}

impl FolderEntry {
    pub fn get(&self) -> String {
        match self {
            FolderEntry::Folder(path) => path.to_owned(),
            FolderEntry::File(name) => name.to_owned(),
        }
    }
}

impl Default for FolderEntry {
    fn default() -> Self {
        FolderEntry::File(String::default())
    }
}

#[derive(Default)]
pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<FolderEntry>,
    pub item_map: HashMap<FolderEntry, String>,
}

impl StatefulList {
    pub fn new() -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
            item_map: HashMap::new(),
        }
    }

    pub fn with_items(
        items: Vec<FolderEntry>,
        item_map: HashMap<FolderEntry, String>,
    ) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
            item_map,
        }
    }

    pub fn with_items_sorted(
        mut items: Vec<FolderEntry>,
        item_map: HashMap<FolderEntry, String>,
    ) -> Self {
        //let mut items: Vec<_> = item_map.iter().map(|s| String::from(s.0)).collect();
        items.sort_by_key(|f| f.get());
        Self::with_items(items, item_map)
    }

    pub fn search(&mut self, query: &str) {
        let mut results: Vec<FolderEntry>;
        results = self
            .item_map
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
        self.items = results;
    }

    pub fn forward(&mut self, amount: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                let loc = i + amount;
                if loc > self.items.len() - 1 {
                    Some(loc - self.items.len())
                } else {
                    Some(loc)
                }
            }
            None => {
                if self.items.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
        };
        self.state.select(i);
    }

    pub fn back(&mut self, amount: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if amount > i {
                    Some(i + self.items.len() - amount)
                } else {
                    Some(i - amount)
                }
            }
            None => {
                if self.items.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
        };
        self.state.select(i);
    }

    pub fn select(&mut self, select: Option<usize>) {
        // Because the state expects item in between the newly selected item and the origin (which aren't always there)
        // we have to select None briefly in order to reset the 'offset' variable in self.state.
        self.state.select(None);
        self.state.select(select);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}
