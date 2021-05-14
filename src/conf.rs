use termion::event::Key;
use tui::style::{Color, Modifier, Style};

pub struct Config {
    pub path: String,
    pub theme: Theme,
    pub keybinds: Keybinds,
    pub auto_select_song: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: String::from("/home/zeus/repos/gpro-rs/"),
            theme: Theme::default(),
            keybinds: Keybinds::default(),
            auto_select_song: false,
        }
    }
}

pub struct Theme {
    pub title: Style,
    pub comment: Style,
    pub chord: Style,
    pub selected: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            comment: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            chord: Style::default().fg(Color::Blue),
            selected: Style::default().fg(Color::Green),
        }
    }
}

pub struct Keybinds {
    pub quit: Key,
    pub search: Key,
}

impl Default for Keybinds {
    fn default() -> Self {
        Keybinds {
            quit: Key::Ctrl('c'),
            search: Key::Char('/'),
        }
    }
}
