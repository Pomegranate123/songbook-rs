use crate::app::App;
use std::cmp::min;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
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

    let searchresults: Vec<ListItem> = app
        .files
        .items
        .iter()
        .map(|s| ListItem::new(s.get()))
        .collect();
    let songlist = List::new(searchresults)
        .block(Block::default().title("Songs").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(songlist, layout[0], &mut app.files.state);

    let input = Text::from(app.input.as_str());
    let search = Paragraph::new(input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if app.searching {
                app.config.theme.selected.to_style()
            } else {
                Style::default()
            })
            .title(Span::from("Search")),
    );

    f.render_widget(search, layout[1]);
}

pub fn draw_song_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    match app.song.as_ref() {
        Some(song) => {
            let song_block = Block::default()
                .title(Span::styled(
                    song.title.as_str(),
                    app.config.theme.title.to_style(),
                ))
                .borders(Borders::ALL);

            let song_rect = song_block.inner(layout_chunk);
            let height = song_rect.height as usize;
            let width = song_rect.width as usize;

            let mut text = song.text.clone();
            let mut columncount = text.len() / height + 1;
            let mut len;

            let mut i = 0;
            while i < 5 {
                i += 1;
                columncount = text.len() / height + 1;
                len = text.len();
                text = text
                    .iter()
                    .flat_map(|line| {
                        if line.width() > width / columncount {
                            let mut spans1 = vec![];
                            let mut spans2 = vec![];
                            let spans = line.clone().0;
                            let mut spanswidth = 0;
                            for span in spans {
                                let spanwidth = span.width();
                                if spanswidth + spanwidth > width && spanswidth < width {
                                    let maxwidth = width - spanswidth;
                                    let mut content = span.content.chars();
                                    spans1.push(Span::styled(
                                        content.by_ref().take(maxwidth).collect::<String>(),
                                        span.style,
                                    ));
                                    spans2
                                        .push(Span::styled(content.collect::<String>(), span.style))
                                } else if spanswidth < width {
                                    spanswidth += span.width();
                                    spans1.push(span);
                                } else {
                                    spans2.push(span);
                                }
                            }
                            vec![Spans::from(spans1), Spans::from(spans2)]
                        } else {
                            vec![line.clone()]
                        }
                    })
                    .collect();
                if len == text.len() {
                    break;
                }
            }

            let constraints: Vec<Constraint> =
                std::iter::repeat(Constraint::Percentage(100 / columncount as u16))
                    .take(columncount)
                    .collect();

            let song_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(constraints.as_ref())
                .split(layout_chunk);

            for (i, column) in song_layout.iter().enumerate() {
                let start = height * i;
                let stop = min(text.len(), height * (i + 1));
                let columntext = text[start..stop].to_owned();
                f.render_widget(Paragraph::new(Text::from(columntext)), *column);
            }
            f.render_widget(song_block, layout_chunk);
        }
        None => f.render_widget(Block::default().borders(Borders::ALL), layout_chunk),
    }
}
