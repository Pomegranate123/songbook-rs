use crate::{app::App, parser::SongLine};
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

    // Format search results into Vec<ListItem>
    let searchresults: Vec<ListItem> = app
        .files
        .items
        .iter()
        .map(|s| ListItem::new(s.get()))
        .collect();

    // Create song list
    let songlist = List::new(searchresults)
        .block(Block::default().title("Songs").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(songlist, layout[0], &mut app.files.state);

    // Only show last characters that fit in search box
    let inner_size = (layout[1].width - 3) as usize; // Two border pixels, one cursor pixel
    let input_length = app.input.chars().count();
    let mut inputtext = &app.input[..];
    if input_length >= inner_size {
        inputtext = &app.input[input_length - inner_size..];
    }

    // Add cursor if search box is selected
    let mut input = vec![Span::from(inputtext)];
    if app.searching {
        input.push(Span::styled("|", app.config.theme.selected.to_style()))
    }

    // Create search box
    let searchbox = Paragraph::new(Text::from(Spans::from(input))).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if app.searching {
                app.config.theme.selected.to_style()
            } else {
                Style::default()
            })
            .title(Span::from("Search")),
    );

    f.render_widget(searchbox, layout[1]);
}

pub fn draw_song_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    match &app.song {
        Some(song) => {
            let song_block = Block::default()
                .title(Span::styled(
                    song.title.as_str(),
                    app.config.theme.title.to_style(),
                ))
                .borders(Borders::ALL);

            let song_rect = song_block.inner(layout_chunk);
            let text = wrap_lines(&song.text, song_rect, app.extra_column_size);

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
                f.render_widget(Paragraph::new(Text::from(text[i].to_spans())), *column);
            }
            f.render_widget(song_block, layout_chunk);
        }
        None => f.render_widget(Block::default().borders(Borders::ALL), layout_chunk),
    }
}

#[derive(Default)]
struct Column<'a> {
    content: Vec<SongLine<'a>>,
}

impl<'a> Column<'a> {
    pub fn from(content: Vec<SongLine<'a>>) -> Self {
        Column { content }
    }

    pub fn width(&self) -> usize {
        self.content
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0)
    }

    pub fn to_spans(&self) -> Vec<Spans<'a>> {
        self.content
            .iter()
            .cloned()
            .flat_map(|line| line.into_spans())
            .collect()
    }
}

fn wrap_lines<'a>(
    text: &[SongLine<'a>],
    container: Rect,
    extra_column_size: usize,
) -> Vec<Column<'a>> {
    let height = (container.height - 2) as usize;

    let mut line_widths = text.iter().map(|line| line.width()).collect::<Vec<usize>>();
    line_widths.sort_unstable();
    let median_width = line_widths[line_widths.len() / 2];

    let line_wrapped_text: Vec<SongLine> = text
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
