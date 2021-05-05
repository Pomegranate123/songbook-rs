mod util;

use crate::util::{
    event::{Event, Events},
    StatefulList,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    cmp::{min, Ordering},
    error::Error,
    fs, io,
};
use termion::{event::Key, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

lazy_static! {
    static ref RE_NEWLINES: Regex = Regex::new(r"(\n\r?|\r\n?)").unwrap();
    static ref RE_TAGS: Regex = Regex::new(r"\{([^\{\}\n]+?)(?::([^\{\}\n]+))?\}\n?").unwrap();
    static ref RE_CHORDS: Regex = Regex::new(r"\[([^\n\[\]]*)\]").unwrap();
    static ref COMMENT_STYLE: Style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    static ref CHORD_STYLE: Style = Style::default().fg(Color::Blue);
    static ref TITLE_STYLE: Style = Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::BOLD);
}

#[derive(Debug, Default, Clone)]
struct Song<'a> {
    title: Option<String>,
    subtitle: Option<String>,
    key: Option<String>,
    text: Vec<Spans<'a>>,
}

impl<'a> Song<'a> {
    fn new(songstring: String) -> Self {
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
                                .push(Spans::from(Span::styled(subtitle, *TITLE_STYLE)));
                            continue 'lines;
                        }
                        "key" => {
                            let key =
                                String::from("Toonsoort ") + cap.get(2).unwrap().as_str().trim();
                            song.key = Some(key.clone());
                            song.text
                                .push(Spans::from(Span::styled(key, *COMMENT_STYLE)));
                            continue 'lines;
                        }
                        "c" => spans.push(Span::styled(
                            String::from(cap.get(2).unwrap().as_str()),
                            *COMMENT_STYLE,
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
                        true => spans.push(Span::styled(String::from(section), *COMMENT_STYLE)),
                        false => Song::parse_chords(&section, &mut chords, &mut spans),
                    },
                }
            }
            if chorus {
                if !spans.is_empty() {
                    spans.insert(0, Span::styled(String::from("| "), *COMMENT_STYLE));
                }
                if !chords.is_empty() {
                    chords.insert(0, Span::styled(String::from("| "), *COMMENT_STYLE));
                }
            }

            if !chords.is_empty() {
                song.text.push(Spans::from(chords));
            }
            song.text.push(Spans::from(spans));
        }
        song
    }

    fn parse_chords(chord_line: &str, chords: &mut Vec<Span<'a>>, spans: &mut Vec<Span<'a>>) {
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
            chords.push(Span::styled(chords_string, *CHORD_STYLE));
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

struct App<'a> {
    items: StatefulList<ListItem<'a>>,
    search: Vec<String>,
    song: Option<Song<'a>>,
    selected: Option<usize>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(
                search()
                    .iter()
                    .map(|s| ListItem::new(String::from(s)))
                    .collect(),
            ),
            search: search(),
            song: None,
            selected: None,
        }
    }

    fn view_song(&mut self, selected: Option<usize>) {
        if self.selected != selected {
            if let Some(index) = selected {
                let songstring = &self.search[index];
                let path = String::from("/home/pomegranate/Dropbox/Songbook/NL Selectie/")
                    + &songstring[..]
                    + ".txt";
                let file = fs::read_to_string(path).unwrap();
                self.song = Some(Song::new(file));
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut term = Terminal::new(backend)?;
    let events = Events::new();

    let mut app = App::new();

    term.clear().unwrap();
    loop {
        let search = app.search.clone();
        let selected = app.items.state.selected();
        app.view_song(selected);

        term.draw(|t| {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Length(20), Constraint::Min(80)].as_ref())
                .split(t.size());

            let searchresults: Vec<ListItem> =
                search.iter().map(|s| ListItem::new(s.as_ref())).collect();
            let songlist = List::new(searchresults)
                .block(Block::default().title("Search").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));

            t.render_stateful_widget(songlist, layout[0], &mut app.items.state);

            let mut song = app.song.clone().unwrap_or_default();
            let song_block = Block::default()
                .title(song.title.unwrap_or_default())
                .borders(Borders::ALL);

            let song_rect = song_block.inner(layout[1]);

            let linecount = song.text.len();
            let height = song_rect.height as usize;

            let columncount = linecount / height + 1;

            let mut constraints = vec![];
            for _ in 0..columncount {
                constraints.push(Constraint::Percentage(100 / columncount as u16))
            }

            let song_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(constraints.as_ref())
                .split(layout[1]);

            for column in song_layout.iter() {
                let song_temp = song.text.split_off(min(height, song.text.len()));
                t.render_widget(Paragraph::new(Text::from(song.text)), *column);
                song.text = song_temp;
            }
            t.render_widget(song_block, layout[1]);
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

fn search() -> Vec<String> {
    let mut paths: Vec<_> = fs::read_dir("/home/pomegranate/Dropbox/Songbook/NL Selectie")
        .unwrap()
        .map(|dir| dir.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    paths
        .iter()
        .filter_map(|f| match f.file_name().into_string() {
            Ok(file) => Some(file.trim_end_matches(".txt").to_string()),
            Err(_) => None,
        })
        .collect()
}
