#![allow(dead_code)]

pub mod event;

use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self, amount: usize) {
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

    pub fn previous(&mut self, amount: usize) {
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
