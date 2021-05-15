extern crate rust_music_theory as rustmt;

use crate::{conf::Theme, util::FolderEntry};
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
}

#[derive(Debug, Default, Clone)]
pub struct Song<'a> {
    pub title: String,
    pub subtitle: String,
    pub songstring: String,
    pub text: Vec<Spans<'a>>,
}

impl<'a> Song<'a> {
    pub fn from(songstring: String, theme: &Theme) -> Self {
        Song::new(songstring, theme, None)
    }

    pub fn in_key(songstring: String, theme: &Theme, key: PitchClass) -> Self {
        Song::new(songstring, theme, Some(key))
    }

    fn new(songstring: String, theme: &Theme, transpose_to: Option<PitchClass>) -> Self {
        let songstring = RE_NEWLINES.replace_all(&songstring, "\n");

        let mut song = Song {
            songstring: songstring.to_string(),
            ..Default::default()
        };

        let mut chorus = false;
        let mut comment = false;
        let mut transposition = 0;
        'lines: for line in songstring.lines() {
            let mut chords: Vec<Span<'a>> = vec![];
            let mut spans = vec![];
            for section in Song::regex_split_keep(&RE_TAGS, &line) {
                match RE_TAGS.captures(&section) {
                    Some(cap) => match cap.get(1).unwrap().as_str() {
                        "t" | "title" => {
                            song.title = String::from(cap.get(2).unwrap().as_str().trim());
                            continue 'lines;
                        }
                        "st" | "subtitle" => {
                            let subtitle = String::from(cap.get(2).unwrap().as_str().trim());
                            song.subtitle = subtitle.clone();
                            song.text
                                .push(Spans::from(Span::styled(subtitle, theme.title)));
                            continue 'lines;
                        }
                        "key" => {
                            if let Some(key) = transpose_to {
                                if let Some(song_key) =
                                    PitchClass::from_str(cap.get(2).unwrap().as_str().trim())
                                {
                                    transposition +=
                                        key.into_u8() as i32 - song_key.into_u8() as i32;
                                }
                            }
                            continue 'lines;
                        }
                        "Capo-Bass_Guitar" => {
                            transposition -=
                                cap.get(2).unwrap().as_str().trim().parse::<i32>().unwrap()
                        }
                        "c" => spans.push(Span::styled(
                            String::from(cap.get(2).unwrap().as_str()),
                            theme.comment,
                        )),
                        "soc" | "start_of_chorus" => {
                            chorus = true;
                            continue 'lines;
                        }
                        "eoc" | "end_of_chorus" => chorus = false,
                        "soh" => comment = true,
                        "eoh" => comment = false,
                        "tag" | "tag:" => continue 'lines,
                        _ => {}
                    },
                    None => match comment {
                        true => spans.push(Span::styled(String::from(section), theme.comment)),
                        false => Song::parse_chords(
                            &section,
                            &theme,
                            &mut chords,
                            &mut spans,
                            transposition,
                        ),
                    },
                }
            }
            if chorus {
                if !spans.is_empty() {
                    spans.insert(0, Span::styled(String::from("| "), theme.comment));
                }
                if !chords.is_empty() {
                    chords.insert(0, Span::styled(String::from("| "), theme.comment));
                }
            }

            if !chords.is_empty() {
                song.text.push(Spans::from(chords));
            }
            song.text.push(Spans::from(spans));
        }
        song
    }

    fn parse_chords(
        chord_line: &str,
        theme: &Theme,
        chords: &mut Vec<Span<'a>>,
        spans: &mut Vec<Span<'a>>,
        transposition: i32,
    ) {
        let mut chords_string = String::new();
        let mut lyrics_string = String::new();
        let chords_width: i32 = chords.iter().map(|s| s.width() as i32).sum();
        let lyrics_width: i32 = spans.iter().map(|s| s.width() as i32).sum();

        for part in Song::regex_split_keep(&RE_CHORDS, &chord_line) {
            match RE_CHORDS.captures(&part) {
                Some(chord) => {
                    let difference = (lyrics_width + lyrics_string.chars().count() as i32)
                        - (chords_width + chords_string.chars().count() as i32);
                    match difference.cmp(&0) {
                        Ordering::Less => {
                            // Chords are longer than lyrics
                            let delimiter = match lyrics_string.chars().last().unwrap_or(' ') {
                                ' ' | ',' | '.' | ':' | ';' => " ",
                                _ => "-",
                            };
                            lyrics_string.push_str(&delimiter.repeat(-difference as usize));
                        }
                        Ordering::Greater => {
                            // Lyrics are longer than chords
                            chords_string.push_str(&" ".repeat(difference as usize));
                        }
                        Ordering::Equal => {}
                    }

                    let chord_tag = chord.get(1).unwrap().as_str();

                    //TODO: Transposition
                    let parsed_chord = RE_ROOT_NOTE.replace_all(chord_tag, |caps: &Captures| {
                        PitchClass::from_interval(
                            PitchClass::from_str(&caps.get(0).unwrap().as_str()).unwrap(),
                            Interval::from_semitone(((transposition + 12) % 12) as u8).unwrap(),
                        )
                        .to_string()
                    });
                    // Interval::from_semitone((transposition % 12) as u8).unwrap(),
                    chords_string.push_str(&parsed_chord);
                    chords_string.push(' ');
                }
                None => lyrics_string.push_str(part),
            }
        }
        if !chords_string.is_empty() {
            chords.push(Span::styled(chords_string, theme.chord));
        }
        if !lyrics_string.is_empty() {
            spans.push(Span::from(lyrics_string));
        }
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
    pub songs: Vec<FolderEntry>,
    pub playliststring: String,
}

impl Playlist {
    pub fn new(playliststring: String) -> Self {
        let mut lines = playliststring.lines();
        Playlist {
            title: lines.next().unwrap().to_string(),
            songs: lines.map(|s| FolderEntry::Song(s.to_string())).collect(),
            playliststring,
        }
    }
}
