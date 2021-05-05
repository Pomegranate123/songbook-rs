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
                    loc - self.items.len()
                } else {
                    loc
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self, amount: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if amount > i {
                    i + self.items.len() - amount
                } else {
                    i - amount
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
