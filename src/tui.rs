use std::{
    io::{stderr, Stderr},
    ops::{Deref, DerefMut},
    time::Duration,
};
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
    use std::time::Instant;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_input_handler_key_event_immediate() {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        // Create a dummy receiver for Tui, keep the real one for the test
        let (_dummy_tx, dummy_rx) = mpsc::unbounded_channel();
        let mut tui =
            Tui::new_with_channels(event_tx.clone(), dummy_rx).expect("Failed to create Tui");
        tui.tick_rate(60.0);
        tui.frame_rate(60.0);
        tui.start();

        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        event_tx.send(Event::Key(key_event)).unwrap();

        let start = Instant::now();
        let received = tokio::time::timeout(std::time::Duration::from_millis(20), async {
            while let Some(event) = event_rx.recv().await {
                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('a') {
                        return true;
                    }
                }
            }
            false
        })
        .await
        .unwrap();

        assert!(received, "Key event was not processed immediately");
        assert!(
            start.elapsed().as_millis() < 20,
            "Key event processing was not immediate"
        );
    }
}

use color_eyre::eyre::Result;
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        cursor,
        event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub type Frame<'a> = ratatui::Frame<'a>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Terminal User Interface (TUI) handler for the application.
///
/// This struct manages the terminal, event channels, and runtime tasks for the application's UI.
pub struct Tui {
    /// The terminal instance used for rendering.
    pub terminal: ratatui::Terminal<CrosstermBackend<Stderr>>,
    /// The background task handle for the TUI event loop.
    pub task: JoinHandle<()>,
    /// Token used to cancel the TUI event loop.
    pub cancellation_token: CancellationToken,
    /// Receiver for incoming UI events.
    pub event_rx: UnboundedReceiver<Event>,
    /// Sender for outgoing UI events.
    pub event_tx: UnboundedSender<Event>,
    /// The frame rate (frames per second) for rendering.
    pub frame_rate: f64,
    /// The tick rate (ticks per second) for event polling.
    pub tick_rate: f64,
}

impl Tui {
    /// Creates a new [`Tui`] instance with default tick and frame rates.
    ///
    /// Initializes the terminal, event channels, and background task for the TUI.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the initialized [`Tui`] instance on success.
    pub fn new() -> Result<Self> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stderr()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
        })
    }
    /// Creates a [`Tui`] instance with externally provided event channels (for testing).
    ///
    /// This method is primarily used for testing, allowing custom event channels to be injected.
    ///
    /// # Arguments
    ///
    /// * `event_tx` - The sender for outgoing [`Event`]s.
    /// * `event_rx` - The receiver for incoming [`Event`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the initialized [`Tui`] instance on success.
    pub fn new_with_channels(
        event_tx: UnboundedSender<Event>,
        event_rx: UnboundedReceiver<Event>,
    ) -> Result<Self> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stderr()))?;
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
        })
    }

    /// Sets the tick rate (ticks per second) for the TUI event loop.
    ///
    /// # Arguments
    ///
    /// * `tick_rate` - The new tick rate in Hertz.
    pub fn tick_rate(&mut self, tick_rate: f64) {
        self.tick_rate = tick_rate;
    }

    /// Sets the frame rate (frames per second) for rendering.
    ///
    /// # Arguments
    ///
    /// * `frame_rate` - The new frame rate in Hertz.
    pub fn frame_rate(&mut self, frame_rate: f64) {
        self.frame_rate = frame_rate;
    }

    /// Starts the TUI event loop and rendering process.
    ///
    /// This method begins polling for events and rendering frames at the configured rates.
    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let _event_tx = self.event_tx.clone();
        // Spawn dedicated input handler
        let input_event_tx = _event_tx.clone();
        let input_cancellation_token = _cancellation_token.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                tokio::select! {
                    _ = input_cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = reader.next() => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        // Only send KeyEvent if it's a fresh Press (not Repeat)
                                        if key.kind == KeyEventKind::Press && key.state == crossterm::event::KeyEventState::NONE {
                                            input_event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        input_event_tx.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    CrosstermEvent::Resize(x, y) => {
                                        input_event_tx.send(Event::Resize(x, y)).unwrap();
                                    },
                                    CrosstermEvent::FocusLost => {
                                        input_event_tx.send(Event::FocusLost).unwrap();
                                    },
                                    CrosstermEvent::FocusGained => {
                                        input_event_tx.send(Event::FocusGained).unwrap();
                                    },
                                    CrosstermEvent::Paste(s) => {
                                        input_event_tx.send(Event::Paste(s)).unwrap();
                                    },
                                }
                            }
                            Some(Err(_)) => {
                                input_event_tx.send(Event::Error).unwrap();
                            }
                            None => {},
                        }
                    }
                }
            }
        });

        // Main loop for tick/render/cancellation
        self.task = tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init).unwrap();
            loop {
                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        break;
                    }
                    _ = tick_interval.tick() => {
                        _event_tx.send(Event::Tick).unwrap();
                    }
                    _ = render_interval.tick() => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        });
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                tracing::error!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<CrosstermBackend<Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
