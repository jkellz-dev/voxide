use std::{collections::HashMap, sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        block, Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState,
        Paragraph, Row, Table,
    },
    Frame,
};
use tokio::{
    sync::{broadcast, mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
};
use tracing::{error, trace};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::Component;
use crate::{
    action::Action,
    config::key_event_to_string,
    errors::Error,
    models::{RadioApi, RadioStation, SearchParam, State},
};

const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;

pub struct StreamState {
    station: RadioStation,
    stream_handle: JoinHandle<()>,
    shutdown_tx: broadcast::Sender<()>,
}

impl StreamState {
    pub fn get_name(&self) -> &str {
        &self.station.name
    }

    pub fn get_url(&self) -> &str {
        &self.station.url
    }

    pub fn shutdown(&self) {
        self.shutdown_tx
            .send(())
            .map_err(|e| -> std::result::Result<usize, _> {
                tracing::error!(error=?e, "failed to send shutdown signal");
                Err(e)
            })
            .unwrap();
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Processing,
}

#[derive(Default)]
pub struct StationsList {
    state: ListState,
    items: Vec<RadioStation>,
    last_selected: Option<usize>,
}

impl StationsList {
    fn new(items: Vec<RadioStation>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return self.state.select(None);
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return self.state.select(None);
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        let offset = self.state.offset();
        self.last_selected = self.state.selected();
        self.state.select(None);
        *self.state.offset_mut() = offset;
    }

    fn select_station(&mut self) -> Option<RadioStation> {
        self.state.selected().map(|i| self.items[i].clone())
    }
}

pub struct Home {
    pub show_help: bool,
    pub radio_api: Arc<RadioApi>,
    pub stations: StationsList,
    pub now_playing: Option<StreamState>,
    throbber_state: throbber_widgets_tui::ThrobberState,
    pub counter: usize,
    pub app_ticker: usize,
    pub render_ticker: usize,
    pub mode: Mode,
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub text: Vec<String>,
}

impl Home {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            radio_api: Arc::new(RadioApi::new().await?),
            stations: Default::default(),
            now_playing: Default::default(),
            throbber_state: Default::default(),
            show_help: Default::default(),
            counter: Default::default(),
            app_ticker: Default::default(),
            render_ticker: Default::default(),
            mode: Default::default(),
            input: Default::default(),
            action_tx: Default::default(),
            keymap: Default::default(),
            text: Default::default(),
        })
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn tick(&mut self) {
        tracing::trace!("Tick");
        self.throbber_state.calc_next();
        self.app_ticker = self.app_ticker.saturating_add(1);
    }

    pub fn render_tick(&mut self) {
        tracing::debug!("Render Tick");
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    pub fn add(&mut self, s: String) {
        self.text.push(s)
    }

    pub fn search_stations(&mut self, params: Vec<SearchParam>) {
        let tx = self.action_tx.clone().unwrap();
        let api = self.radio_api.clone();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tracing::info!(?params, "Searching stations");
            let stations = api.get_stations(params).await.unwrap();

            tx.send(Action::StationsFound(stations)).unwrap();
            tx.send(Action::ExitProcessing).unwrap();
        });
    }

    pub fn apply_stations(&mut self, stations: Vec<RadioStation>) {
        self.stations = StationsList::new(stations);
    }

    pub fn next_item(&mut self) {
        self.stations.next();
    }

    pub fn previous_item(&mut self) {
        self.stations.previous();
    }

    pub fn select_station(&mut self) {
        if let Some(station) = self.stations.select_station() {
            if let Some(now_paying) = self.now_playing.as_ref() {
                if station.stationuuid == now_paying.station.stationuuid {
                    self.stop_station();
                    return;
                }
            }
            self.play_station(station);
        }
    }

    fn play_station(&mut self, station: RadioStation) {
        if let Some(tx) = &self.action_tx {
            let mut play_station = station.clone();

            let (shutdown_tx, mut _shutdown_rx) = broadcast::channel(1);
            let download_shutdown_rx = shutdown_tx.subscribe();
            let play_shutdown_rx = shutdown_tx.subscribe();
            let handle = tokio::spawn(async move {
                tracing::info!("Starting play");
                play_station
                    .play(download_shutdown_rx, play_shutdown_rx)
                    .await
                    .unwrap();
                tracing::info!("Done playing");
            });

            self.now_playing = Some(StreamState {
                station,
                stream_handle: handle,
                shutdown_tx,
            });

            tx.send(Action::ExitProcessing).unwrap();
        }
    }

    pub fn stop_station(&mut self) {
        if let Some(state) = self.now_playing.as_ref() {
            state.shutdown_tx.send(()).expect("failed to send stop");
            state.stream_handle.abort()
        }
        self.now_playing = None;
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    // fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    //     let action = match self.mode {
    //         Mode::Normal | Mode::Processing => return Ok(None),
    //         Mode::Insert => match key.code {
    //             KeyCode::Esc => Action::EnterNormal,
    //             KeyCode::Enter => {
    //                 if let Some(sender) = &self.action_tx {
    //                     if let Err(e) =
    //                         sender.send(Action::CompleteInput(self.input.value().to_string()))
    //                     {
    //                         error!("Failed to send action: {:?}", e);
    //                     }
    //                 }
    //                 Action::EnterNormal
    //             }
    //             _ => {
    //                 self.input.handle_event(&crossterm::event::Event::Key(key));
    //                 Action::Update
    //             }
    //         },
    //     };
    //     Ok(Some(action))
    // }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::ToggleShowHelp => self.show_help = !self.show_help,
            Action::NextItem => self.next_item(),
            Action::PreviousItem => self.previous_item(),
            Action::Search(s) => self.search_stations(s),
            Action::StationsFound(stations) => self.apply_stations(stations),
            Action::PlaySelectedStation => self.select_station(),
            Action::StopPlayingStation => self.stop_station(),
            // Action::StreamStarted(station) => self.start_stream(station),
            Action::EnterNormal => {
                self.mode = Mode::Normal;
            }
            Action::EnterInsert => {
                self.mode = Mode::Insert;
            }
            Action::EnterProcessing => {
                self.mode = Mode::Processing;
            }
            Action::ExitProcessing => {
                // TODO: Make this go to previous mode instead
                self.mode = Mode::Normal;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let mut min = 3;

        if self.show_help {
            min = 4
        }

        let rects = Layout::default()
            .constraints(
                [
                    Constraint::Min(3),
                    Constraint::Percentage(100),
                    Constraint::Min(min),
                ]
                .as_ref(),
            )
            .split(rect);

        // TOP
        // NOW PLAYING
        let mut lines = vec![];
        let throbber = throbber_widgets_tui::Throbber::default()
            .throbber_style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
            .use_type(throbber_widgets_tui::WhichUse::Spin)
            .to_symbol_span(&self.throbber_state);

        let now_playing_block = Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![Span::raw("Now Playing ")]))
            .bg(NORMAL_ROW_COLOR);

        if let Some(radio_station) = self.now_playing.as_ref() {
            lines.push(Line::from(vec![
                throbber,
                Span::styled(
                    radio_station.get_name().to_owned(),
                    Style::default().fg(Color::Red),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "Nothing...",
                Style::default().fg(Color::Yellow),
            )]));
        };

        let np_widget = Paragraph::new(lines).block(now_playing_block);

        f.render_widget(np_widget, rects[0]);

        let inner_block = Block::new()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .stations
            .items
            .iter()
            .enumerate()
            .map(|(i, station)| station.to_list_item(i))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        f.render_stateful_widget(items, rects[1], &mut self.stations.state);

        // BOTTOM
        if self.mode == Mode::Insert {
            f.set_cursor(
                (rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2),
                rects[1].y + 1,
            )
        }

        let width = rects[1].width.max(5) - 3; // keep 2 for borders and 1 for cursor
        let mut lines = vec![];

        let mut help_block = Block::default().borders(Borders::ALL).bg(NORMAL_ROW_COLOR);
        let spacer = Span::raw("   ");

        let default_help = Line::from(vec![
            Span::styled(
                "k",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("Up", Style::default().fg(Color::DarkGray)),
            spacer.clone(),
            Span::styled(
                "j",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("Down", Style::default().fg(Color::DarkGray)),
            spacer.clone(),
            Span::styled(
                "/",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("search", Style::default().fg(Color::DarkGray)),
            spacer.clone(),
            Span::styled(
                "enter",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("play/stop", Style::default().fg(Color::DarkGray)),
            spacer.clone(),
            Span::styled(
                "q",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("quit", Style::default().fg(Color::DarkGray)),
            spacer.clone(),
            Span::styled(
                "?",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled("More", Style::default().fg(Color::DarkGray)),
        ]);

        if self.show_help {
            help_block = help_block.title("Help");
            lines.push(Line::from(vec![
                Span::styled(
                    "tab",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Gray),
                ),
                Span::raw(" "),
                Span::styled("Next Field", Style::default().fg(Color::DarkGray)),
                spacer.clone(),
            ]));
            lines.push(default_help);
        } else {
            lines.push(default_help);
        }

        let help_widget = Paragraph::new(lines).block(help_block);

        f.render_widget(help_widget, rects[2]);

        // f.render_widget(
        //     Block::default()
        //         .title(
        //             ratatui::widgets::block::Title::from(format!(
        //                 "{:?}",
        //                 &self
        //                     .last_events
        //                     .iter()
        //                     .map(key_event_to_string)
        //                     .collect::<Vec<_>>()
        //             ))
        //             .alignment(Alignment::Right),
        //         )
        //         .title_style(Style::default().add_modifier(Modifier::BOLD)),
        //     Rect {
        //         x: rect.x + 1,
        //         y: rect.height.saturating_sub(1),
        //         width: rect.width.saturating_sub(2),
        //         height: 1,
        //     },
        // );

        Ok(())
    }
}
