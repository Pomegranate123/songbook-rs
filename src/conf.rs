#![allow(dead_code)]

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
    pub extra_column_size: usize,
    pub column_padding: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            path: String::from("."),
            theme: Theme::default(),
            keybinds: Keybinds::default(),
            auto_select_song: false,
            extra_column_size: 15,
            column_padding: 2,
        }
    }
}

impl Config {
    pub fn load(file: &std::path::Path) -> Result<Config, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(file)?;
        Ok(serde_yaml::from_str(&contents)?)
    }

    pub fn write_default(file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if file.exists() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "File already exists",
            )));
        }

        let conf = Config::default();
        let contents = serde_yaml::to_string(&conf)?;
        std::fs::write(file, &contents)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Theme {
    pub title: ConfStyle,
    pub comment: ConfStyle,
    pub chord: ConfStyle,
    pub selected: ConfStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title: ConfStyle::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            comment: ConfStyle::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            chord: ConfStyle::default().fg(Color::Blue),
            selected: ConfStyle::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    pub up: SerDeKey,
    pub down: SerDeKey,
    pub next: SerDeKey,
    pub back: SerDeKey,
    pub jump_up: SerDeKey,
    pub jump_down: SerDeKey,
    pub col_size_inc: SerDeKey,
    pub col_size_dec: SerDeKey,
    pub search: SerDeKey,
    pub quit: SerDeKey,
}

impl Default for Keybinds {
    fn default() -> Self {
        Keybinds {
            up: SerDeKey(Key::Up),
            down: SerDeKey(Key::Down),
            next: SerDeKey(Key::Right),
            back: SerDeKey(Key::Left),
            jump_up: SerDeKey(Key::PageUp),
            jump_down: SerDeKey(Key::PageDown),
            col_size_inc: SerDeKey(Key::End),
            col_size_dec: SerDeKey(Key::Home),
            search: SerDeKey(Key::Char('/')),
            quit: SerDeKey(Key::Ctrl('c')),
        }
    }
}

/// Style replacement which uses SerDeModifier in order to be readable when serialized
#[derive(Serialize, Deserialize)]
pub struct ConfStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Vec<SerDeModifier>,
}

impl ConfStyle {
    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.push(SerDeModifier(modifier));
        self
    }

    pub fn to_style(&self) -> Style {
        self.modifiers.iter().fold(
            Style {
                fg: self.fg,
                bg: self.bg,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            },
            |style, m| style.add_modifier(**m),
        )
    }
}

impl Default for ConfStyle {
    fn default() -> Self {
        ConfStyle {
            fg: None,
            bg: None,
            modifiers: vec![],
        }
    }
}

/// Termion key wrapper that has serialize and deserialize
pub struct SerDeKey(Key);

impl std::ops::Deref for SerDeKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for SerDeKey {
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

impl<'de> Deserialize<'de> for SerDeKey {
    fn deserialize<D>(deserializer: D) -> Result<SerDeKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = SerDeKey;

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
            s if s.starts_with('F') && s.len() >= 2 => {
                let n: u8 = match s[1..].parse() {
                    Ok(num) => num,
                    Err(_) => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
                };
                Key::F(n)
            }
            "Null" => Key::Null,
            "Esc" => Key::Esc,
            s if s.len() == 1 => Key::Char(s.chars().next().unwrap()),
            s if s.starts_with("Alt+") && s.len() == 5 => {
                let c = s.chars().nth(4).unwrap();
                Key::Alt(c)
            }
            s if s.starts_with("Ctrl+") && s.len() == 6 => {
                let c = s.chars().nth(5).unwrap();
                Key::Ctrl(c)
            }
            _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
        };

        Ok(SerDeKey(key))
    }
}

/// Tui Modifier wrapper that has readable serialize and deserialize
pub struct SerDeModifier(Modifier);

impl std::ops::Deref for SerDeModifier {
    type Target = Modifier;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for SerDeModifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = match self.0 {
            Modifier::BOLD => "Bold",
            Modifier::DIM => "Dim",
            Modifier::ITALIC => "Italic",
            Modifier::UNDERLINED => "Underlined",
            Modifier::SLOW_BLINK => "Slow blink",
            Modifier::RAPID_BLINK => "Rapid blink",
            Modifier::REVERSED => "Reversed",
            Modifier::HIDDEN => "Hidden",
            Modifier::CROSSED_OUT => "Strikethrough",
            _ => unreachable!(),
        };
        serializer.serialize_str(&string)
    }
}

impl<'de> Deserialize<'de> for SerDeModifier {
    fn deserialize<D>(deserializer: D) -> Result<SerDeModifier, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ModifierVisitor)
    }
}

struct ModifierVisitor;

impl<'de> Visitor<'de> for ModifierVisitor {
    type Value = SerDeModifier;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string containing a valid modifier")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let modifier = match s {
            "Bold" => Modifier::BOLD,
            "Dim" => Modifier::DIM,
            "Italic" => Modifier::ITALIC,
            "Underlined" => Modifier::UNDERLINED,
            "Slow blink" => Modifier::SLOW_BLINK,
            "Rapid blink" => Modifier::RAPID_BLINK,
            "Reversed" => Modifier::REVERSED,
            "Hidden" => Modifier::HIDDEN,
            "Strikethrough" => Modifier::CROSSED_OUT,
            _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
        };

        Ok(SerDeModifier(modifier))
    }
}
