#![feature(iter_intersperse)]
mod app;
mod conf;
mod parser;
mod ui;
mod util;

use crate::{
    app::App,
    conf::Config,
    util::event::{self, Event, Events},
};
use std::{error::Error, io, time::Duration};
use termion::{event::Key, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);

    let mut term = Terminal::new(backend)?;
    let events = Events::with_config(event::Config {
        exit_key: Key::Ctrl('c'),
        tick_rate: Duration::from_millis(250),
    });

    let mut app = App::new(Config::default());

    term.clear().unwrap();
    loop {
        term.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(20), Constraint::Min(80)].as_ref())
                .split(f.size());

            ui::draw_search_list(f, &mut app, layout[0]);
            ui::draw_song_block(f, &app, layout[1]);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                _ if key == app.config.keybinds.search && !app.searching => app.searching = true,
                _ if key == app.config.keybinds.quit => break,
                Key::Char(c) => {
                    if app.searching {
                        app.input.push(c);
                        app.files.items = app.search(&app.input);
                        let len_results = app.files.items.len();
                        if let Some(index) = app.files.selected() {
                            if index >= len_results {
                                if app.files.items.is_empty() {
                                    app.files.select(None);
                                } else {
                                    app.files.select(Some(len_results - 1));
                                }
                            }
                        }
                    } else if key == app.config.keybinds.search {
                        app.searching = true;
                    }
                }
                Key::Down => {
                    app.files.forward(1);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                }
                Key::Up => {
                    app.files.back(1);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                }
                Key::PageDown => {
                    app.files.forward(20);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                }
                Key::PageUp => {
                    app.files.back(20);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                }
                Key::Backspace => {
                    if app.searching {
                        app.input.pop();
                        app.files.items = app.search(&app.input);
                    }
                }
                Key::Right => app.load_selected(),
                Key::Left => app.path_back(),
                Key::Esc => app.searching = false,
                _ => (),
            },
            Event::Tick => (),
        }
    }
    Ok(())
}
