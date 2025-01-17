use crate::tui::App;
use crate::keyboard::{*};

use ratatui::{
    Frame,
    symbols::border,
    widgets::{
        Block,
        block::Title,
        block::Position,
        Borders,
        Paragraph
    },
    prelude::*,
    widgets::*,
};

impl App {
    pub fn render_search(&mut self, app_container: Rect, frame: &mut Frame) {
        // search bar up top, results in 3 lists. Artists, Albums, Tracks
        // split the app container into 2 parts
        let search_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(3),
                Constraint::Percentage(95),
            ])
            .split(app_container);

        let search_area = search_layout[0];
        let results_area = search_layout[1];

        // render search bar
        if self.searching {
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(style::Color::Blue),
                search_area,
            );
        } else {
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(style::Color::Gray),
                search_area,
            );
        };

        // search term
        let search_term = Paragraph::new(self.search_term.clone())
            .block(Block::default().borders(Borders::ALL).title("Search Term"))
            .wrap(Wrap { trim: false });
        frame.render_widget(search_term, search_area);

        let instructions = if self.searching {
            Title::from(Line::from(vec![
                " Search ".white().into(),
                "<Enter>".blue().bold(),
                " Clear search ".white().into(),
                "<Delete>".blue().bold(),
                " Cancel ".white().into(),
                "<Esc> ".blue().bold(),
            ]))
        } else {
            Title::from(Line::from(vec![
                " Go ".white().into(),
                "<Enter>".blue().bold(),
                " Search ".white().into(),
                "< / > <F2>".blue().bold(),
                " Next Section ".white().into(),
                "<Tab>".blue().bold(),
                " Previous Section ".white().into(),
                "<Shift+Tab> ".blue().bold(),
            ]))
        };

        Block::default()
            .title("Search")
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .render(search_area, frame.buffer_mut());

        // split results area into 3 parts
        let results_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(results_area);

        // render search results
        // 3 lists, artists, albums, tracks
        let artists = self
            .search_result_artists
            .iter()
            .map(|artist| artist.name.as_str())
            .collect::<Vec<&str>>();

        let albums = self
            .search_result_albums
            .iter()
            .map(|album| album.name.as_str())
            .collect::<Vec<&str>>();
        let tracks = self
            .search_result_tracks
            .iter()
            .map(|track| {
                let title = format!("{} - {}", track.name, track.album);
                // track.run_time_ticks is in microseconds
                let seconds = (track.run_time_ticks / 1_000_0000) % 60;
                let minutes = (track.run_time_ticks / 1_000_0000 / 60) % 60;
                let hours = (track.run_time_ticks / 1_000_0000 / 60) / 60;
                let hours_optional_text = match hours {
                    0 => String::from(""),
                    _ => format!("{}:", hours),
                };

                let mut time_span_text = format!("  {}{:02}:{:02}", hours_optional_text, minutes, seconds);
                if track.has_lyrics{
                    time_span_text.push_str(" (l)");
                }
                if track.id == self.active_song_id {
                    let mut time: Text = Text::from(title);
                    time.push_span(
                        Span::styled(
                            time_span_text,
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )
                    );
                    ListItem::new(time)
                        .style(Style::default().fg(Color::Blue))
                } else {
                    let mut time: Text = Text::from(title);
                    time.push_span(
                        Span::styled(
                            time_span_text,
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )
                    );
                    ListItem::new(time)
                }
            })
            .collect::<Vec<ListItem>>();

        let artists_list = match self.search_section {
            SearchSection::Artists => List::new(artists)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(style::Color::Blue)
                        .title("Artists")
                )
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                )
                .scroll_padding(10)
                .repeat_highlight_symbol(true),
            _ => List::new(artists)
                .block(Block::default().borders(Borders::ALL).title("Artists"))
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                )
                .scroll_padding(10)
                .repeat_highlight_symbol(true),
        };

        let albums_list = match self.search_section {
            SearchSection::Albums => List::new(albums)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(style::Color::Blue)
                        .title("Albums")
                )
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                )
                .repeat_highlight_symbol(true),
            _ => List::new(albums)
                .block(Block::default().borders(Borders::ALL).title("Albums"))
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                )
                .repeat_highlight_symbol(true),
        };

        let tracks_list = match self.search_section {
            SearchSection::Tracks => List::new(tracks)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(style::Color::Blue)
                        .title("Tracks")
                )
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                )
                .repeat_highlight_symbol(true),
            _ => List::new(tracks)
                .block(Block::default().borders(Borders::ALL).title("Tracks"))
                .highlight_symbol(">>")
                .highlight_style(
                    Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
                    .fg(Color::Black)
                )
                .repeat_highlight_symbol(true),
        };

        // frame.render_widget(artists_list, results_layout[0]);
        frame.render_stateful_widget(artists_list, results_layout[0], &mut self.selected_search_artist);
        frame.render_stateful_widget(albums_list, results_layout[1], &mut self.selected_search_album);
        frame.render_stateful_widget(tracks_list, results_layout[2], &mut self.selected_search_track);

        // render search results
    }
}