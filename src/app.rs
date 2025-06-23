use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{fps::FpsCounter, home::Home, search::Search, Component},
    config::Config,
    mode::Mode,
    tui,
};

pub struct App {
    /// Application configuration settings.
    pub config: Config,
    /// The interval (in Hz) at which the application's tick events occur.
    pub tick_rate: f64,
    /// The interval (in Hz) at which the application's frames are rendered.
    pub frame_rate: f64,
    /// The list of UI components managed by the application.
    pub components: Vec<Box<dyn Component>>,
    /// Indicates whether the application should quit.
    pub should_quit: bool,
    /// Indicates whether the application should suspend (e.g., for shelling out).
    pub should_suspend: bool,
    /// The current mode of the application.
    pub mode: Mode,
    /// Key events received during the last tick.
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    /// Creates a new instance of [`App`] with the specified tick and frame rates.
    ///
    /// # Arguments
    ///
    /// * `tick_rate` - The interval (in Hz) at which the application's tick events occur.
    /// * `frame_rate` - The interval (in Hz) at which the application's frames are rendered.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the component initializations or configuration loading fails.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the initialized [`App`] instance on success.
    pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let home = Home::new().await?;
        let fps = FpsCounter::default();
        let search = Search::default();
        let config = Config::new()?;
        let mode = Mode::Home;
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home), Box::new(search), Box::new(fps)],
            should_quit: false,
            should_suspend: false,
            config,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    /// Runs the main application loop, handling events and updating the UI.
    ///
    /// This method initializes the TUI, processes actions, and manages the application's
    /// event-driven workflow until termination is requested.
    ///
    /// # Errors
    ///
    /// Returns an error if the TUI fails to initialize or if any event loop operation fails.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] indicating success or failure of the application run.
    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?;

        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                tracing::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single key action,
                                // then consider it for multi-key combinations.
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    tracing::info!("Got action: {action:?}");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        };
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    tracing::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Mode(mode) => self.mode = mode,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?;

                tui.tick_rate(self.tick_rate);
                tui.frame_rate(self.frame_rate);
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
