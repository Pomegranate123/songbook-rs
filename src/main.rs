mod app;
mod conf;
mod song;
mod ui;
mod util;

use crate::{
    app::App,
    util::event::{Config, Event, Events},
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
    let events = Events::with_config(Config {
        exit_key: Key::Esc,
        tick_rate: Duration::from_millis(250),
    });

    let mut app = App::new();

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
                Key::Esc => break,
                Key::Down => {
                    app.songlist.next(1);
                    app.load_song(app.songlist.selected());
                }
                Key::PageDown => {
                    app.songlist.next(20);
                    app.load_song(app.songlist.selected());
                }
                Key::Up => {
                    app.songlist.previous(1);
                    app.load_song(app.songlist.selected());
                }
                Key::PageUp => {
                    app.songlist.previous(20);
                    app.load_song(app.songlist.selected());
                }
                Key::Char(c) => {
                    app.input.push(c);
                    app.songlist.items = app.search_songs(&app.input);
                    let len_results = app.songlist.items.len();
                    if let Some(index) = app.songlist.selected() {
                        if index >= len_results {
                            if app.songlist.items.is_empty() {
                                app.songlist.select(None);
                            } else {
                                app.songlist.select(Some(len_results - 1));
                            }
                        }
                    }
                }
                Key::Backspace => {
                    app.input.pop();
                    app.songlist.items = app.search_songs(&app.input);
                }
                _ => (),
            },
            Event::Tick => (),
        }
    }
    Ok(())
}
