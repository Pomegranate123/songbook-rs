mod app;
mod conf;
mod song;
mod ui;
mod util;

use crate::{
    app::App,
    util::event::{Event, Events},
};
use std::{error::Error, io};
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
    let events = Events::new();

    let mut app = App::new();

    term.clear().unwrap();
    loop {
        let selected = app.items.state.selected();
        app.load_song(selected);

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
                Key::Char('q') => break,
                Key::Down => app.items.next(1),
                Key::PageDown => app.items.next(20),
                Key::Up => app.items.previous(1),
                Key::PageUp => app.items.previous(20),
                _ => (),
            },
            Event::Tick => (),
        }
    }
    Ok(())
}
