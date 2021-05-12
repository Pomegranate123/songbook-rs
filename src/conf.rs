use tui::style::{Color, Modifier, Style};

pub struct Config {
    pub path: String,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: String::from("/home/pomegranate/Dropbox/Songbook"),
            theme: Theme::default(),
        }
    }
}

pub struct Theme {
    pub title: Style,
    pub comment: Style,
    pub chord: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            comment: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            chord: Style::default().fg(Color::Blue),
        }
    }
}
