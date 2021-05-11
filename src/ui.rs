use crate::app::App;
use std::cmp::min;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_search_list<B>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(100), Constraint::Length(3)])
        .split(layout_chunk);

    let list = app.songlist.items.clone();
    let searchresults: Vec<ListItem> = list.iter().map(|s| ListItem::new(s.as_ref())).collect();
    let songlist = List::new(searchresults)
        .block(Block::default().title("Songs").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(songlist, layout[0], &mut app.songlist.state);

    let input = Text::from(app.input.clone());
    let search = Paragraph::new(input).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::from("Search")),
    );

    f.render_widget(search, layout[1]);
}

pub fn draw_song_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let mut song = app.song.clone().unwrap_or_default();
    let song_block = Block::default()
        .title(Span::styled(song.title, app.config.theme.title))
        .borders(Borders::ALL);

    let song_rect = song_block.inner(layout_chunk);

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
        .split(layout_chunk);

    for column in song_layout.iter() {
        let song_temp = song.text.split_off(min(height, song.text.clone().len()));
        f.render_widget(Paragraph::new(Text::from(song.text)), *column);
        song.text = song_temp;
    }
    f.render_widget(song_block, layout_chunk);
}
