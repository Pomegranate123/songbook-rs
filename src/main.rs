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
use getopts::Options;
use std::{error::Error, io, time::Duration};
use termion::{event::Key, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse commandline arguments
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("c", "config", "set config file", "CONFIG");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "default-config", "write the default config");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(program, opts);
        return Ok(());
    }

    if matches.opt_present("default-config") && matches.opt_present("c") {
        let path = std::path::PathBuf::from(matches.opt_str("c").unwrap());
        Config::write_default(&path)?;
        println!("Default config has been written to {}", path.display());
        return Ok(());
    }

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);

    let mut term = Terminal::new(backend)?;
    let events = Events::with_config(event::Config {
        exit_key: Key::Ctrl('c'),
        tick_rate: Duration::from_millis(250),
    });

    // If a config file is supplied load it otherwise use default settings
    let mut app = match matches.opt_str("c") {
        Some(config) => {
            let path = std::path::PathBuf::from(&config);
            if !path.exists() {
                panic!("Path '{}' doesn't exist", config)
            }
            App::new(Config::load(&path)?)
        }
        None => App::new(Config::default()),
    };

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
            Event::Input(key) => {
                if app.searching {
                    match key {
                        Key::Char(c) => {
                            app.input.push(c);
                            app.files.items = app.search(&app.input);
                            app.files.select(None);
                        }
                        Key::Backspace => {
                            app.input.pop();
                            app.files.items = app.search(&app.input);
                        }
                        Key::Esc => app.searching = false,
                        _ => (),
                    }
                } else if key == *app.config.keybinds.down {
                    app.files.forward(1);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                } else if key == *app.config.keybinds.up {
                    app.files.back(1);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                } else if key == *app.config.keybinds.jump_down {
                    app.files.forward(20);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                } else if key == *app.config.keybinds.jump_up {
                    app.files.back(20);
                    if app.config.auto_select_song {
                        app.load_selected()
                    }
                } else if key == *app.config.keybinds.next {
                    app.load_selected()
                } else if key == *app.config.keybinds.back {
                    app.path_back()
                } else if key == *app.config.keybinds.search {
                    app.searching = true
                } else if key == *app.config.keybinds.quit {
                    break;
                };
            }
            Event::Tick => (),
        }
    }
    Ok(())
}
