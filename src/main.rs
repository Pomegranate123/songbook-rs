mod app;
mod conf;
mod parser;
mod ui;
mod util;

use crate::{
    app::{App, AppState},
    conf::Config,
    util::{Event, Events},
};
use getopts::Options;
use std::{env, error::Error, io, time::Duration};
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
    opts.optopt("c", "config", "set config file", "PATH");
    opts.optopt("", "default-config", "write the default config", "PATH");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f)
        }
    };

    if matches.opt_present("h") {
        print_usage(program, opts);
        return Ok(());
    }

    if let Some(arg) = matches.opt_str("default-config") {
        let path = std::path::PathBuf::from(&arg);
        Config::write_default(&path)?;
        println!("Default config has been written to {}", path.display());
        return Ok(());
    }

    let config = match matches.opt_str("c") {
        Some(arg) => {
            let path = std::path::PathBuf::from(&arg);
            if !path.exists() {
                panic!("Path '{}' doesn't exist", arg)
            }
            Config::load(&path)?
        }
        None => {
            let config_path = env::var("GPRO_CONFIG")
                .unwrap_or_else(|_| String::from("/home/pomegranate/git/gpro-rs/conf.yml"));
            Config::load(&std::path::PathBuf::from(config_path)).unwrap_or_default()
        }
    };

    let mut app = App::new(config.clone());

    if matches.opt_present("d") {
        return Ok(());
    }

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);

    let mut term = Terminal::new(backend)?;
    let events = Events::with_config(util::Config {
        exit_key: *config.keybinds.quit,
        tick_rate: Duration::from_millis(250),
    });

    term.clear().unwrap();
    loop {
        term.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(20), Constraint::Min(80)].as_ref())
                .split(f.size());

            let left_bar = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Max(100), Constraint::Length(3)])
                .split(layout[0]);

            match app.state {
                AppState::Default => ui::draw_song_list(f, &mut app, layout[0]),
                AppState::Searching => {
                    ui::draw_song_list(f, &mut app, left_bar[0]);
                    ui::draw_search_bar(f, &mut app, left_bar[1]);
                }
                AppState::Transposing => {
                    ui::draw_song_list(f, &mut app, left_bar[0]);
                    ui::draw_transposition(f, &mut app, left_bar[1]);
                }
            }
            ui::draw_song(f, &app, layout[1]);
        })?;

        match events.next()? {
            Event::Input(key) => {
                if key == *app.config.keybinds.quit {
                    break;
                }
                match app.state {
                    AppState::Default => {
                        if key == *app.config.keybinds.search {
                            app.state = AppState::Searching
                        } else if key == *app.config.keybinds.transpose {
                            app.state = AppState::Transposing;
                        }
                    }
                    AppState::Searching => {
                        if key == Key::Esc {
                            app.state = AppState::Default
                        }
                        match key {
                            Key::Char(c) => match c {
                                '\n' => (),
                                _ => {
                                    app.input.push(c);
                                    app.search();
                                }
                            },
                            Key::Backspace => {
                                app.input.pop();
                                app.search();
                            }
                            _ => (),
                        }
                    }
                    AppState::Transposing => {
                        if key == Key::Esc {
                            app.state = AppState::Default
                        } else if key == *app.config.keybinds.search {
                            app.state = AppState::Searching
                        }
                    }
                }
                if key == *app.config.keybinds.down {
                    app.get_nav_mut().forward(1);
                    if app.config.auto_select_song {
                        app.load_selected_song()
                    }
                } else if key == *app.config.keybinds.up {
                    app.get_nav_mut().back(1);
                    if app.config.auto_select_song {
                        app.load_selected_song()
                    }
                } else if key == *app.config.keybinds.jump_down {
                    app.get_nav_mut().forward(20);
                    if app.config.auto_select_song {
                        app.load_selected_song()
                    }
                } else if key == *app.config.keybinds.jump_up {
                    app.get_nav_mut().back(20);
                    if app.config.auto_select_song {
                        app.load_selected_song()
                    }
                } else if key == *app.config.keybinds.next {
                    app.load_selected()
                } else if key == *app.config.keybinds.back {
                    app.get_nav_mut().path_back()
                } else if key == *app.config.keybinds.col_size_inc {
                    app.config.extra_column_size += 1;
                } else if key == *app.config.keybinds.col_size_dec
                    && app.config.extra_column_size > 0
                {
                    app.config.extra_column_size -= 1;
                }
            }
            Event::Tick => (),
        }
    }
    Ok(())
}
