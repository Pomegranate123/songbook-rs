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
    match app.song.as_ref() {
        Some(song) => {
            let song_block = Block::default()
                .title(Span::styled(
                    song.title.as_str(),
                    app.config.theme.title.to_style(),
                ))
                .borders(Borders::ALL);

            let song_rect = song_block.inner(layout_chunk);
            let text = wrap_lines(&song.text, song_rect, song_rect.width as usize);

            let constraints: Vec<Constraint> = text
                .iter()
                .map(|column| Constraint::Length(column.width() as u16))
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

pub fn draw_test<B>(f: &mut Frame<B>, _app: &App, container: Rect)
where
    B: Backend,
{
    use crate::{conf::Theme, parser::Song};

    let song_block = Block::default().borders(Borders::ALL);

    let song_rect = container;
    let text = Song::from(
        String::from(
            "
[|][F]Zelfs de blinden [|][C]zien uw heerschap[|][G]pij [F][|][G]
{c:Refrein}
[|][F]Hoor [C]ons ge[|][G]bed en [|][C]kom! [⁄][⁄][⁄][|][C][⁄][⁄][⁄]

",
        ),
        &Theme::default(),
    )
    .text;
    let text = wrap_lines(&text, song_rect, song_rect.width as usize);
    let height = (song_rect.height - 2) as usize;
    let columncount = text.len() / height + 1;

    let constraints: Vec<Constraint> =
        std::iter::repeat(Constraint::Percentage(100 / columncount as u16))
            .take(columncount)
            .collect();

    let song_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(constraints.as_ref())
        .split(song_rect);

    for (i, column) in song_layout.iter().enumerate() {
        f.render_widget(Paragraph::new(Text::from(text[i].to_spans())), *column);
    }
    f.render_widget(song_block, song_rect);
}

#[derive(Default)]
struct Column<'a> {
    content: Vec<SongLine<'a>>,
}

impl<'a> Column<'a> {
    pub fn from(content: Vec<SongLine<'a>>) -> Self {
        Column { content }
    }

    pub fn height(&self) -> usize {
        self.content.iter().map(|line| line.height()).sum()
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
            .flat_map(|line| line.to_spans())
            .collect()
    }
}

fn wrap_lines<'a>(
    text: &[SongLine<'a>],
    container: Rect,
    max_column_width: usize,
) -> Vec<Column<'a>> {
    let height = (container.height - 2) as usize;
    let width = (container.width - 2) as usize;

    let wrapped_text = text.iter().for_each(|line| {});

    let line_wrapped_text: Vec<SongLine> = text
        .iter()
        .flat_map(|line| {
            if line.width() > max_column_width {
                let split_line = line.split_at(max_column_width).unwrap();
                vec![split_line.0, split_line.1]
            } else {
                vec![line.clone()]
            }
        })
        .collect();

    let mut column_wrapped_text: Vec<Column> = vec![];
    let mut columnheight = 0;
    let mut rest = text;
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

    let total_width = column_wrapped_text
        .iter()
        .map(|column| column.width())
        .sum::<usize>();
    if total_width > width {
        column_wrapped_text = wrap_lines(text, container, max_column_width - 1)
    }

    column_wrapped_text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{conf::Theme, parser::Song};

    #[test]
    fn linewrap() {
        let text = Song::from(
            String::from(
                "
[|][F]Zelfs de blinden [|][C]zien uw heerschap[|][G]pij [F][|][G]
{c:Refrein}
[|][F]Hoor [C]ons ge[|][G]bed en [|][C]kom! [⁄][⁄][⁄][|][C][⁄][⁄][⁄]

",
            ),
            &Theme::default(),
        )
        .text;
        let container = Rect::new(0, 0, 20, 20);
        let wrappedtext = wrap_lines(&text, container, container.width as usize);
        //println!("{:#?}", wrappedtext);
    }
}
