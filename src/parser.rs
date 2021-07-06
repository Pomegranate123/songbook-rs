extern crate rust_music_theory as rustmt;

use crate::{conf::Theme, util::FileType};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use rustmt::{interval::Interval, note::PitchClass};
use std::cmp::Ordering;
use tui::text::{Span, Spans};

lazy_static! {
    static ref RE_NEWLINES: Regex = Regex::new(r"(\n\r?|\r\n?)").unwrap();
    static ref RE_TAGS: Regex = Regex::new(r"\{([^\{\}\n]+?)(?::([^\{\}\n]+))?\}\n?").unwrap();
    static ref RE_CHORDS: Regex = Regex::new(r"\[([^\n\[\]]*)\]").unwrap();
    static ref RE_ROOT_NOTE: Regex = Regex::new(r"[ABCDEFG][b#]?").unwrap();
    static ref RE_SPACES: Regex = Regex::new(r" +").unwrap();
    static ref RE_BLOCKS: Regex = Regex::new(r"[^ \n]+ *").unwrap();
}

#[derive(Debug, Clone)]
pub enum SongString {
    Chord(String),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct SongBlock(Vec<SongString>);

impl SongBlock {
    pub fn from(input: &str, transposition: i32) -> Self {
        SongBlock(
            Song::regex_split_keep(&RE_CHORDS, input)
                .iter()
                .map(|part| match RE_CHORDS.captures(&part) {
                    Some(chord) => {
                        let chord = chord.get(1).unwrap().as_str();
                        let transposed = RE_ROOT_NOTE.replace_all(chord, |caps: &Captures| {
                            PitchClass::from_interval(
                                PitchClass::from_str(caps.get(0).unwrap().as_str()).unwrap(),
                                Interval::from_semitone(((transposition + 12) % 12) as u8).unwrap(),
                            )
                            .to_string()
                        });
                        SongString::Chord(transposed.to_string())
                    }
                    None => SongString::Text(part.to_string()),
                })
                .collect(),
        )
    }

    pub fn from_comment(c: &str) -> Self {
        SongBlock(vec![SongString::Comment(c.to_owned())])
    }

    pub fn width(&self) -> usize {
        let mut chords: usize = 0;
        let mut text: usize = 0;
        self.0.iter().for_each(|songstring| match songstring {
            SongString::Chord(c) => {
                match text.cmp(&chords) {
                    Ordering::Less => text = chords,
                    Ordering::Greater => chords = text,
                    Ordering::Equal => (),
                }
                chords += c.chars().count() + 1;
            }
            SongString::Text(t) => {
                text += t.chars().count();
            }
            SongString::Comment(c) => {
                text += c.chars().count();
            }
        });
        std::cmp::max(chords, text)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SongLine {
    blocks: Vec<SongBlock>,
    chorus: bool,
}

impl SongLine {
    pub fn from(blocks: Vec<SongBlock>, chorus: bool) -> Self {
        SongLine { blocks, chorus }
    }

    pub fn width(&self) -> usize {
        self.format(&Theme::default())
            .iter()
            .map(|s| s.width())
            .max()
            .unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.format(&Theme::default()).len()
    }

    pub fn format<'a>(&self, theme: &Theme) -> Vec<Spans<'a>> {
        let mut has_chords = false;
        let mut chords: Vec<Span<'a>> = vec![];
        let mut text: Vec<Span<'a>> = vec![];
        if self.chorus {
            chords.push(Span::styled("| ", theme.comment.to_style()));
            text.push(Span::styled("| ", theme.comment.to_style()));
        }
        self.blocks.iter().for_each(|block| {
            block.0.iter().for_each(|songstring| match songstring {
                SongString::Chord(c) => {
                    has_chords = true;
                    let text_len: usize = text.iter().map(Span::width).sum();
                    let chords_len: usize = chords.iter().map(Span::width).sum();
                    match text_len.cmp(&chords_len) {
                        Ordering::Equal => (),
                        Ordering::Less => {
                            let delimiter = match text.iter().last() {
                                Some(span) => match span.content.chars().last().unwrap_or(' ') {
                                    ' ' | ',' | '.' | ':' | ';' => " ",
                                    _ => "-",
                                },
                                None => " ",
                            };
                            text.push(Span::from(delimiter.repeat(chords_len - text_len)))
                        }
                        Ordering::Greater => {
                            chords.push(Span::from(" ".repeat(text_len - chords_len)))
                        }
                    }
                    chords.push(Span::styled(c.to_owned() + " ", theme.chord.to_style()));
                }
                SongString::Text(t) => {
                    text.push(Span::styled(t.to_owned(), theme.lyrics.to_style()));
                }
                SongString::Comment(c) => {
                    text.push(Span::styled(c.to_owned(), theme.comment.to_style()));
                }
            })
        });
        let mut formatted = vec![];
        if has_chords {
            formatted.push(Spans::from(chords))
        }
        formatted.push(Spans::from(text));
        formatted
    }

    pub fn wrap(&self, max_width: usize) -> Vec<Self> {
        if max_width >= self.width() {
            return vec![self.clone()];
        }
        let chorus_width = match self.chorus {
            true => 2,
            false => 0,
        };

        let mut total_width = 0;
        let mut wrapped_line = vec![];
        let mut wrapped_lines = vec![];

        for block in self.blocks.iter() {
            let block_width = block.width();

            if total_width + block_width + chorus_width < max_width {
                wrapped_line.push(block.clone());
                total_width += block.width();
            } else {
                wrapped_lines.push(SongLine::from(wrapped_line, self.chorus));
                wrapped_line = vec![block.clone()];
                total_width = block.width();
            }
        }
        wrapped_lines.push(SongLine::from(wrapped_line, self.chorus));

        wrapped_lines
    }
}

#[derive(Debug, Default, Clone)]
pub struct Song {
    pub title: String,
    pub subtitle: String,
    pub transposition: i32,
    pub key: Option<PitchClass>,
    pub content: Vec<SongLine>,
}

impl Song {
    pub fn from(songstring: String) -> Self {
        Song::new(songstring, None)
    }

    pub fn in_key(songstring: String, key: PitchClass) -> Self {
        Song::new(songstring, Some(key))
    }

    fn new(songstring: String, key: Option<PitchClass>) -> Self {
        let songstring = RE_NEWLINES.replace_all(&songstring, "\n");
        let songstring = RE_SPACES.replace_all(&songstring, " ");

        let mut song = Song {
            key,
            ..Default::default()
        };

        let mut chorus = false;
        let mut comment = false;
        for line in songstring.lines() {
            let mut tag = false;
            let mut blocks: Vec<SongBlock> = vec![];
            for section in Song::regex_split_keep(&RE_TAGS, &line) {
                match RE_TAGS.captures(&section) {
                    Some(cap) => {
                        tag = true;
                        match cap.get(1).unwrap().as_str() {
                            "t" | "title" => {
                                song.title = String::from(cap.get(2).unwrap().as_str().trim());
                            }
                            "st" | "subtitle" => {
                                song.subtitle = String::from(cap.get(2).unwrap().as_str().trim());
                            }
                            "key" => {
                                let original_key =
                                    PitchClass::from_str(cap.get(2).unwrap().as_str().trim());
                                match song.key {
                                    Some(display_key) => {
                                        song.transposition += display_key.into_u8() as i32
                                            - original_key
                                                .expect("Error parsing key tag in song")
                                                .into_u8()
                                                as i32
                                    }
                                    None => song.key = original_key,
                                }
                            }
                            "Capo-Bass_Guitar" => {
                                song.transposition -=
                                    cap.get(2).unwrap().as_str().trim().parse::<i32>().unwrap()
                            }
                            "c" => blocks
                                .append(&mut Song::parse_comment(cap.get(2).unwrap().as_str())),
                            "soc" | "start_of_chorus" => {
                                chorus = true;
                            }
                            "eoc" | "end_of_chorus" => {
                                chorus = false;
                            }
                            "soh" => comment = true,
                            "eoh" => comment = false,
                            _ => (),
                        }
                    }
                    None => match comment {
                        true => blocks.append(&mut Song::parse_comment(section)),
                        false => blocks.append(&mut Song::parse_line(section, song.transposition)),
                    },
                }
            }
            if !blocks.is_empty() || !tag {
                song.content.push(SongLine::from(blocks, chorus));
            }
        }
        song
    }

    fn parse_comment(input: &str) -> Vec<SongBlock> {
        RE_BLOCKS
            .captures_iter(input)
            .map(|cap| SongBlock::from_comment(cap.get(0).unwrap().as_str()))
            .collect()
    }

    fn parse_line(input: &str, transposition: i32) -> Vec<SongBlock> {
        RE_BLOCKS
            .captures_iter(input)
            .map(|cap| SongBlock::from(cap.get(0).unwrap().as_str(), transposition))
            .collect()
    }

    fn regex_split_keep<'b>(re: &Regex, text: &'b str) -> Vec<&'b str> {
        let mut result = Vec::new();
        let mut last = 0;
        for (index, matched) in text.match_indices(re) {
            if last != index {
                result.push(&text[last..index]);
            }
            result.push(matched);
            last = index + matched.len();
        }
        if last < text.len() {
            result.push(&text[last..]);
        }
        result
    }

    pub fn get_name(songstring: &str) -> String {
        let songstring = RE_NEWLINES.replace_all(songstring, "\n");
        let mut title = String::from("Untitled");
        let mut subtitle = String::new();

        for line in songstring.lines() {
            for section in Song::regex_split_keep(&RE_TAGS, &line) {
                if let Some(cap) = RE_TAGS.captures(&section) {
                    match cap.get(1).unwrap().as_str() {
                        "t" | "title" => title = String::from(cap.get(2).unwrap().as_str().trim()),
                        "st" | "subtitle" => {
                            subtitle = String::from(cap.get(2).unwrap().as_str().trim())
                        }
                        _ => {}
                    }
                }
            }
        }
        if subtitle.is_empty() {
            return title;
        }
        format!("{} - {}", title, subtitle)
    }
}

pub struct Playlist {
    pub title: String,
    pub songs: Vec<FileType>,
    pub playliststring: String,
}

impl Playlist {
    pub fn from(playliststring: String) -> Self {
        let mut lines = playliststring.lines();
        Playlist {
            title: lines.next().unwrap().to_string(),
            songs: lines.map(|s| FileType::Song(s.to_string())).collect(),
            playliststring,
        }
    }
}
