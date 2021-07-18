use crate::{
    app::{App, FileType},
    conf::Theme,
    parser::*,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_song_list<B>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    // Format search results into Vec<ListItem>
    let searchresults: Vec<ListItem> = app
        .get_nav()
        .current()
        .files
        .iter()
        .map(|file| {
            ListItem::new(Spans::from(match file {
                FileType::Folder(_) => Span::styled(
                    app.config.icons.folder.clone() + &file.name(),
                    app.config.theme.folder.to_style(),
                ),
                FileType::Song(_) => Span::styled(
                    app.config.icons.song.clone() + &file.name(),
                    app.config.theme.song.to_style(),
                ),
                FileType::Playlist(_) => Span::styled(
                    app.config.icons.playlist.clone() + &file.name(),
                    app.config.theme.playlist.to_style(),
                ),
            }))
        })
        .collect();

    // Create song list
    let songlist = List::new(searchresults)
        .block(
            Block::default()
                .title(app.get_nav().current().name.clone())
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(
        songlist,
        layout_chunk,
        &mut app.get_nav_mut().current_mut().state,
    );
}

pub fn draw_search_bar<B>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    // Only show last characters that fit in search box
    let inner_size = (layout_chunk.width - 3) as usize; // Two border pixels, one cursor pixel
    let input = &app.input;
    let input_length = input.chars().count();
    let mut inputtext = &input[..];
    if input_length >= inner_size {
        inputtext = &input[input_length - inner_size..];
    }

    // Add cursor if search box is selected
    let input = vec![
        Span::from(inputtext),
        Span::styled("|", app.config.theme.selected.to_style()),
    ];

    // Create search box
    let searchbox = Paragraph::new(Text::from(Spans::from(input))).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(app.config.theme.selected.to_style())
            .title(Span::from("Search")),
    );

    f.render_widget(searchbox, layout_chunk);
}

pub fn draw_transposition<B>(f: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    let transpose_block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.config.theme.selected.to_style())
        .title(Span::from("Transpose"));

    let transpose = Paragraph::new(Text::from(match &app.song {
        Some(song) => match song.key {
            Some(key) => key.to_string(),
            None => String::from("No key found"),
        },
        None => String::from("No song selected"),
    }))
    .block(transpose_block);
    //    match &app.song {
    //        Some(song) => {
    //            let key_block = List::new()
    //        }
    //        None => (),
    //    }
    f.render_widget(transpose, layout_chunk)
}

pub fn draw_song<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    match &app.song {
        Some(song) => {
            let song_block = Block::default()
                .title(Span::styled(
                    format!("{} - {}", song.title.as_str(), song.subtitle.as_str()),
                    app.config.theme.title.to_style(),
                ))
                .borders(Borders::ALL);

            let song_rect = song_block.inner(layout_chunk);
            let text = wrap_lines(&song.content, song_rect, app.config.extra_column_size);

            let constraints: Vec<Constraint> = text
                .iter()
                .map(|column| {
                    Constraint::Length(column.width() as u16 + app.config.column_padding as u16)
                })
                .collect();

            let song_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(constraints.as_ref())
                .split(layout_chunk);

            for (i, column) in song_layout.iter().enumerate() {
                f.render_widget(
                    Paragraph::new(Text::from(text[i].to_spans(&app.config.theme))),
                    *column,
                );
            }
            f.render_widget(song_block, layout_chunk);
        }
        None => f.render_widget(Block::default().borders(Borders::ALL), layout_chunk),
    }
}

#[derive(Debug, Default)]
pub struct Column {
    content: Vec<SongLine>,
}

impl<'a> Column {
    pub fn from(content: Vec<SongLine>) -> Self {
        Column { content }
    }

    pub fn width(&self) -> usize {
        self.content
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0)
    }

    pub fn to_spans(&self, theme: &Theme) -> Vec<Spans<'a>> {
        self.content
            .iter()
            .cloned()
            .flat_map(|line| line.format(theme))
            .collect()
    }
}

pub fn wrap_lines(lines: &[SongLine], container: Rect, extra_column_size: usize) -> Vec<Column> {
    let height = (container.height - 2) as usize;
    if lines.is_empty() {
        return vec![];
    }

    let mut line_widths: Vec<usize> = lines.iter().map(|line| line.width()).collect();
    line_widths.sort_unstable();
    let median_width = line_widths[line_widths.len() / 2];

    let line_wrapped_text: Vec<SongLine> = lines
        .iter()
        .flat_map(|line| line.wrap(median_width + extra_column_size))
        .collect();

    let mut column_wrapped_text: Vec<Column> = vec![];
    let mut columnheight = 0;
    let mut rest = &line_wrapped_text[..];
    let mut i = 0;
    line_wrapped_text.iter().for_each(|line| {
        if columnheight + line.height() > height {
            let split_text = rest.split_at(i);
            rest = split_text.1;
            column_wrapped_text.push(Column::from(split_text.0.to_vec()));
            columnheight = 0;
            i = 0;
        } else {
            columnheight += line.height();
            i += 1;
        }
    });
    column_wrapped_text.push(Column::from(rest.to_vec()));

    column_wrapped_text
}
