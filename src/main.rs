#![feature(iter_intersperse)]

mod app;
mod conf;
mod parser;
mod ui;
mod util;

use crate::{
    app::{App, AppState},
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

            match app.state {
                AppState::Songs => ui::draw_search_list(f, &mut app, layout[0]),
                AppState::Playlists => ui::draw_playlists(f, &mut app, layout[0]),
                AppState::Playlist => ui::draw_playlist(f, &mut app, layout[0]),
            }
            ui::draw_song_block(f, &app, layout[1]);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    if app.searching {
                        app.input.push(c);
                        let list = match app.state {
                            AppState::Songs => &mut app.songs,
                            AppState::Playlists => &mut app.playlists,
                            AppState::Playlist => &mut app.playlist,
                        };
                        list.search(&app.input);
                        let len_results = list.items.len();
                        if let Some(index) = list.selected() {
                            if index >= len_results {
                                if list.items.is_empty() {
                                    list.select(None);
                                } else {
                                    list.select(Some(len_results - 1));
                                }
                            }
                        }
                    } else {
                        match c {
                            '/' => app.searching = true,
                            'p' => app.state = AppState::Playlists,
                            's' => app.state = AppState::Songs,
                            _ => {}
                        }
                    }
                }
                Key::Down | Key::PageDown | Key::Up | Key::PageUp => {
                    let list = match app.state {
                        AppState::Songs => &mut app.songs,
                        AppState::Playlists => &mut app.playlists,
                        AppState::Playlist => &mut app.playlist,
                    };
                    match key {
                        Key::Down => list.forward(1),
                        Key::PageDown => list.forward(20),
                        Key::Up => list.back(1),
                        Key::PageUp => list.back(20),
                        _ => (),
                    }
                    if app.config.auto_select_song {
                        match app.state {
                            AppState::Songs => app.load(),
                            AppState::Playlists => (),
                            AppState::Playlist => app.load(),
                        }
                    }
                }
                Key::Backspace => {
                    if app.searching {
                        app.input.pop();
                        match app.state {
                            AppState::Songs => app.songs.search(&app.input),
                            AppState::Playlists => app.playlists.search(&app.input),
                            AppState::Playlist => app.playlist.search(&app.input),
                        }
                    }
                }
                Key::Right => app.load(),
                Key::Left => match app.state {
                    AppState::Songs => app.path_back(),
                    AppState::Playlists => (),
                    AppState::Playlist => app.state = AppState::Playlists,
                },
                Key::Esc => app.searching = false,
                Key::Ctrl('c') => break,
                _ => (),
            },
            Event::Tick => (),
        }
    }
    Ok(())
}
