use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    Frame,
};
use tokio::signal;

use crate::{
    errors::Error,
    models::{RadioApi, RadioStation},
    tui,
};

pub struct App {
    api: RadioApi,
    stations: Vec<RadioStation>,
    counter: i8,
    exit: bool,
}

impl App {
    pub async fn new() -> Result<Self, Error> {
        let api = RadioApi::new().await?;
        Ok(Self {
            api,
            stations: Vec::default(),
            counter: 0,
            exit: false,
        })
    }
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('/') => self.get_stations().await,
            KeyCode::Enter => self.play_kexp().await,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    async fn get_stations(&mut self) {
        // TODO: deal with error
        let stations = self
            .api
            .get_stations()
            .await
            .expect("Failed to get stations");

        self.stations = stations;
    }

    async fn play_kexp(&mut self) {
        println!("https://kexp-mp3-128.streamguys1.com/kexp128.mp3");

        let url = "https://kexp-mp3-128.streamguys1.com/kexp128.mp3";

        let mut station = RadioStation::new(url, "kexp");

        let _guard = station.play().await.expect("Failed to play station");

        // station.sink.unwrap().sleep_until_end();

        tokio::select! {
            _ = signal::ctrl_c() => {},
        }
    }

    // fn increment_counter(&mut self) {
    //     self.counter += 1;
    // }
    //
    // fn decrement_counter(&mut self) {
    //     self.counter -= 1;
    // }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Voxide ".bold());
        let instructions = Title::from(Line::from(vec![
            " Search ".into(),
            "</>".blue().bold(),
            " Play ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let stations: Vec<_> = self
            .stations
            .iter()
            .map(|s| Line::from(vec!["Station: ".into(), s.name.clone().yellow()]))
            .collect();

        let station_text = Text::from(stations);

        Paragraph::new(station_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {

    use ratatui::style::Style;

    use super::*;

    #[tokio::test]
    async fn render() {
        let app = App::new().await.unwrap();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[tokio::test]
    async fn handle_key_event() -> io::Result<()> {
        let mut app = App::new().await.unwrap();
        app.handle_key_event(KeyCode::Right.into()).await;
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into()).await;
        assert_eq!(app.counter, 0);

        let mut app = App::new().await.unwrap();
        app.handle_key_event(KeyCode::Char('q').into()).await;
        assert!(app.exit);

        Ok(())
    }
}
