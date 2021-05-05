use crate::conf::Theme;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use tui::text::{Span, Spans};

lazy_static! {
    static ref RE_NEWLINES: Regex = Regex::new(r"(\n\r?|\r\n?)").unwrap();
    static ref RE_TAGS: Regex = Regex::new(r"\{([^\{\}\n]+?)(?::([^\{\}\n]+))?\}\n?").unwrap();
    static ref RE_CHORDS: Regex = Regex::new(r"\[([^\n\[\]]*)\]").unwrap();
}

#[derive(Debug, Default, Clone)]
pub struct Song<'a> {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub key: Option<String>,
    pub text: Vec<Spans<'a>>,
}

impl<'a> Song<'a> {
    pub fn new(songstring: String, theme: &Theme) -> Self {
        let songstring = RE_NEWLINES.replace_all(&songstring, "\n");

        let mut song = Song::default();

        let mut chorus = false;
        let mut comment = false;
        'lines: for line in songstring.lines() {
            let mut chords: Vec<Span<'a>> = vec![];
            let mut spans = vec![];
            for section in Song::regex_split_keep(&RE_TAGS, &line) {
                match RE_TAGS.captures(&section) {
                    Some(cap) => match cap.get(1).unwrap().as_str() {
                        "t" | "title" => {
                            song.title = Some(String::from(cap.get(2).unwrap().as_str().trim()));
                            continue 'lines;
                        }
                        "st" | "subtitle" => {
                            let subtitle = String::from(cap.get(2).unwrap().as_str().trim());
                            song.subtitle = Some(subtitle.clone());
                            song.text
                                .push(Spans::from(Span::styled(subtitle, theme.title)));
                            continue 'lines;
                        }
                        "key" => {
                            let key =
                                String::from("Toonsoort ") + cap.get(2).unwrap().as_str().trim();
                            song.key = Some(key.clone());
                            song.text
                                .push(Spans::from(Span::styled(key, theme.comment)));
                            continue 'lines;
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
                        false => Song::parse_chords(&section, &theme, &mut chords, &mut spans),
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
                            let mut delimiter = '-';
                            match lyrics_string.chars().last() {
                                Some(' ') | Some(',') | Some('.') | Some(':') | Some(';')
                                | None => delimiter = ' ',
                                _ => (),
                            }
                            for _ in 0..-difference {
                                lyrics_string.push(delimiter);
                            }
                        }
                        Ordering::Greater => {
                            // Lyrics are longer than chords
                            for _ in 0..difference {
                                chords_string.push(' ');
                            }
                        }
                        Ordering::Equal => {}
                    }

                    let chord_tag = chord.get(1).unwrap().as_str();
                    chords_string.push_str(chord_tag);
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
}
