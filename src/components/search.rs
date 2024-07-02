use std::collections::HashMap;
use std::time::Instant;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::Component;
use crate::mode::Mode as AppMode;
use crate::models::SearchParam;
use crate::{action::Action, tui::Frame};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum InputMode {
    #[default]
    None,
    Name,
    Country,
    Language,
    Tags,
    Limit,
    Order,
    Reverse,
}

#[derive(Debug, Clone)]
pub struct Search {
    pub action_tx: Option<UnboundedSender<Action>>,
    show_search: bool,
    pub keymap: HashMap<KeyEvent, Action>,
    input_mode: InputMode,
    search_name: Input,
    search_country: Input,
    search_language: Input,
    search_tags: Input,
    search_limit: Input,
    search_order: Input,
    search_reverse: Input,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            show_search: false,
            keymap: Default::default(),
            input_mode: Default::default(),
            search_name: Default::default(),
            search_country: Default::default(),
            search_language: Default::default(),
            search_tags: Default::default(),
            search_limit: Default::default(),
            search_order: Default::default(),
            search_reverse: Default::default(),
        }
    }

    fn get_search_param(&self) -> Vec<SearchParam> {
        let mut result = Vec::new();

        if !self.search_name.value().is_empty() {
            result.push(SearchParam::Name(self.search_name.value().to_string()))
        };

        if !self.search_country.value().is_empty() {
            result.push(SearchParam::Country(
                self.search_country.value().to_string(),
            ))
        };

        if !self.search_language.value().is_empty() {
            result.push(SearchParam::Language(
                self.search_language.value().to_string(),
            ))
        };

        if !self.search_tags.value().is_empty() {
            let tags = self
                .search_tags
                .value()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            result.push(SearchParam::Tags(tags))
        };

        result
    }

    fn send_search_params(&self) -> Action {
        let params = self.get_search_param();
        tracing::info!(?params, "sending search");
        if let Some(sender) = &self.action_tx {
            if let Err(e) = sender.send(Action::Search(params)) {
                tracing::error!("Failed to send action: {:?}", e);
            }
        }
        Action::HomeMode
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    fn tick(&mut self) {}

    fn render_tick(&mut self) {}
}

impl Component for Search {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let mut result = None;
        match action {
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::SearchMode => {
                self.show_search = true;
                self.input_mode = InputMode::Name;
                result = Some(Action::Mode(AppMode::Search));
            }
            Action::HomeMode => {
                self.show_search = false;
                self.input_mode = InputMode::None;
                result = Some(Action::Mode(AppMode::Home));
            }
            _ => (),
        }
        Ok(result)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.input_mode {
            InputMode::None => return Ok(None),
            InputMode::Name => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Country;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Reverse;
                    Action::Update
                }
                _ => {
                    self.search_name
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Country => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Language;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Name;
                    Action::Update
                }
                _ => {
                    self.search_country
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Language => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Tags;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Country;
                    Action::Update
                }
                _ => {
                    self.search_language
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Tags => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Limit;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Language;
                    Action::Update
                }
                _ => {
                    self.search_tags
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Limit => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Order;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Tags;
                    Action::Update
                }
                _ => {
                    self.search_tags
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Order => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Reverse;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Limit;
                    Action::Update
                }
                _ => {
                    self.search_tags
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::Reverse => match key.code {
                KeyCode::Enter => self.send_search_params(),
                KeyCode::Tab => {
                    self.input_mode = InputMode::Name;
                    Action::Update
                }
                KeyCode::BackTab => {
                    self.input_mode = InputMode::Order;
                    Action::Update
                }
                _ => {
                    self.search_tags
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
        };
        Ok(Some(action))
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        if self.show_search {
            let rect = rect.inner(Margin {
                horizontal: 10,
                vertical: 10,
            });

            let wrapper = Layout::new(Direction::Vertical, [Constraint::Max(20)])
                .horizontal_margin(2)
                .vertical_margin(1)
                .split(rect);

            let layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ],
            )
            .split(wrapper[0]);

            let first_row =
                Layout::new(Direction::Horizontal, [Constraint::Percentage(100)]).split(layout[0]);

            let second_row = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Percentage(50),
                    Constraint::Min(0),
                    Constraint::Percentage(50),
                ],
            )
            .split(layout[1]);

            let third_row =
                Layout::new(Direction::Horizontal, [Constraint::Percentage(100)]).split(layout[2]);

            let fourth_row = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Percentage(33),
                    Constraint::Min(0),
                    Constraint::Percentage(34),
                    Constraint::Min(0),
                    Constraint::Percentage(33),
                ],
            )
            .split(layout[3]);

            f.render_widget(Clear, rect);

            let block = Block::default().title(Line::from(vec![Span::styled(
                "Search",
                Style::default().add_modifier(Modifier::BOLD),
            )]));
            f.render_widget(block, rect);

            let width = first_row[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
            let name_block = Paragraph::new(self.search_name.value())
                .style(match self.input_mode {
                    InputMode::Name => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_name.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "name",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(name_block, first_row[0]);

            let country_block = Paragraph::new(self.search_country.value())
                .style(match self.input_mode {
                    InputMode::Country => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_country.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "country",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(country_block, second_row[0]);

            let language_block = Paragraph::new(self.search_language.value())
                .style(match self.input_mode {
                    InputMode::Language => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_language.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "language",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(language_block, second_row[2]);

            let tags_block = Paragraph::new(self.search_tags.value())
                .style(match self.input_mode {
                    InputMode::Tags => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_tags.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "tags",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(tags_block, third_row[0]);

            let limit_block = Paragraph::new(self.search_limit.value())
                .style(match self.input_mode {
                    InputMode::Limit => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_limit.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "limit",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );
            f.render_widget(limit_block, fourth_row[0]);

            let order_block = Paragraph::new(self.search_order.value())
                .style(match self.input_mode {
                    InputMode::Order => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_order.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "order",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(order_block, fourth_row[2]);

            let reverse_block = Paragraph::new(self.search_reverse.value())
                .style(match self.input_mode {
                    InputMode::Reverse => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .scroll((0, self.search_reverse.visual_scroll(width as usize) as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![Span::styled(
                            "reverse",
                            Style::default().add_modifier(Modifier::BOLD),
                        )])),
                );

            f.render_widget(reverse_block, fourth_row[4]);

            let width = first_row[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

            match self.input_mode {
                InputMode::None =>
                    // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                    {}

                InputMode::Name => {
                    let scroll = self.search_name.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        first_row[0].x
                            + ((self.search_name.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        first_row[0].y + 1,
                    )
                }
                InputMode::Country => {
                    let scroll = self.search_country.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        second_row[0].x
                            + ((self.search_country.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        second_row[0].y + 1,
                    )
                }
                InputMode::Language => {
                    let scroll = self.search_language.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        second_row[2].x
                            + ((self.search_language.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        second_row[2].y + 1,
                    )
                }
                InputMode::Tags => {
                    let scroll = self.search_tags.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        third_row[0].x
                            + ((self.search_tags.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        third_row[0].y + 1,
                    )
                }
                InputMode::Limit => {
                    let scroll = self.search_limit.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        fourth_row[0].x
                            + ((self.search_limit.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        fourth_row[0].y + 1,
                    )
                }
                InputMode::Order => {
                    let scroll = self.search_order.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        fourth_row[2].x
                            + ((self.search_order.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        fourth_row[2].y + 1,
                    )
                }
                InputMode::Reverse => {
                    let scroll = self.search_reverse.visual_scroll(width as usize);
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        fourth_row[4].x
                            + ((self.search_reverse.visual_cursor()).max(scroll) - scroll) as u16
                            + 1,
                        // Move one line down, from the border to the input line
                        fourth_row[4].y + 1,
                    )
                }
            }
        };

        Ok(())
    }
}
