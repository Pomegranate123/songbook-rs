use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use termion::event::Key;
use tui::style::{Color, Modifier, Style};

#[derive(Serialize, Deserialize)]
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

impl Config {
    fn load(file: &std::path::Path) -> Result<Config, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(file)?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    pub quit: TerKey,
    pub search: TerKey,
}

impl Default for Keybinds {
    fn default() -> Self {
        Keybinds {
            quit: TerKey(Key::Ctrl('c')),
            search: TerKey(Key::Char('/')),
        }
    }
}

/// Termion key wrapper that has serialize and deserialize
pub struct TerKey(Key);

impl std::ops::Deref for TerKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for TerKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = match self.0 {
            Key::Backspace => "Backspace".to_string(),
            Key::Left => "Left".to_string(),
            Key::Right => "Right".to_string(),
            Key::Up => "Up".to_string(),
            Key::Down => "Down".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::BackTab => "BackTab".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::F(n) => format!("F{}", n),
            Key::Char(c) => c.to_string(),
            Key::Alt(c) => format!("Alt+{}", c),
            Key::Ctrl(c) => format!("Ctrl+{}", c),
            Key::Null => "Null".to_string(),
            Key::Esc => "Esc".to_string(),
            Key::__IsNotComplete => unreachable![],
        };
        serializer.serialize_str(&string)
    }
}

impl<'de> Deserialize<'de> for TerKey {
    fn deserialize<D>(deserializer: D) -> Result<TerKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = TerKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string containing a valid keycode")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let key = match s {
            "Backspace" => Key::Backspace,
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Up" => Key::Up,
            "Down" => Key::Down,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "BackTab" => Key::BackTab,
            "Delete" => Key::Delete,
            "Insert" => Key::Insert,
            s if s.starts_with("F") && s.len() == 2 => {
                // FIXME unwraps
                let n = s
                    .chars()
                    .skip(1)
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<u8>()
                    .unwrap();
                Key::F(n)
            }
            "Null" => Key::Null,
            "Esc" => Key::Esc,
            s if s.len() == 1 => Key::Char(s.chars().next().unwrap()),
            s if s.starts_with("Alt+") && s.len() == 5 => {
                // FIXME unwraps
                let c = s.chars().skip(1).next().unwrap();
                Key::Alt(c)
            }
            s if s.starts_with("Ctrl+") && s.len() == 6 => {
                // FIXME unwraps
                let c = s.chars().skip(1).next().unwrap();
                Key::Ctrl(c)
            }
            _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
        };

        Ok(TerKey(key))
    }
}
