use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    config::Config,
    tui::{Event, Frame},
};

pub mod fps;
pub mod home;
pub mod search;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to receive events,
/// update state, and be rendered on the screen.
pub trait Component {
    /// Registers an action handler for the component.
    ///
    /// This method allows the component to send [`Action`]s for processing via the provided channel.
    /// Override this method if your component needs to emit actions to the main application loop.
    ///
    /// # Arguments
    ///
    /// * `tx` - An [`UnboundedSender`] used to send [`Action`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler registration fails.
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    /// Registers a configuration handler for the component.
    ///
    /// This method allows the component to receive configuration settings from the application.
    /// Override this method if your component needs to be configured at runtime.
    ///
    /// # Arguments
    ///
    /// * `config` - The [`Config`] settings to apply to the component.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler registration fails.
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        Ok(())
    }
    /// Initializes the component with a specified area.
    ///
    /// Override this method if your component needs to perform setup or layout calculations
    /// based on its assigned rectangular area.
    ///
    /// # Arguments
    ///
    /// * `area` - The [`Rect`] representing the area to initialize the component within.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    fn init(&mut self, area: Rect) -> Result<()> {
        Ok(())
    }
    /// Handles incoming events and produces actions if necessary.
    ///
    /// This method processes the provided event and may return an [`Action`] to be handled by the application.
    /// Override this method to implement custom event handling logic for your component.
    ///
    /// # Arguments
    ///
    /// * `event` - An [`Option<Event>`] representing the event to be processed.
    ///
    /// # Errors
    ///
    /// Returns an error if event handling fails.
    ///
    /// # Returns
    ///
    /// Returns an [`Option<Action>`] to be processed, or `None` if no action is produced.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }
    /// Handles key events and produces actions if necessary.
    ///
    /// This method processes the provided key event and may return an [`Action`] to be handled by the application.
    /// Override this method to implement custom key event handling logic for your component.
    ///
    /// # Arguments
    ///
    /// * `key` - The [`KeyEvent`] to be processed.
    ///
    /// # Errors
    ///
    /// Returns an error if key event handling fails.
    ///
    /// # Returns
    ///
    /// Returns an [`Option<Action>`] to be processed, or `None` if no action is produced.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Handles mouse events and produces actions if necessary.
    ///
    /// This method processes the provided mouse event and may return an [`Action`] to be handled by the application.
    /// Override this method to implement custom mouse event handling logic for your component.
    ///
    /// # Arguments
    ///
    /// * `mouse` - The [`MouseEvent`] to be processed.
    ///
    /// # Errors
    ///
    /// Returns an error if mouse event handling fails.
    ///
    /// # Returns
    ///
    /// Returns an [`Option<Action>`] to be processed, or `None` if no action is produced.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Updates the state of the component based on a received action. (REQUIRED)
    ///
    /// This method processes the provided [`Action`] and may update the component's state or produce a new action.
    /// Override this method to implement custom state update logic for your component.
    ///
    /// # Arguments
    ///
    /// * `action` - The [`Action`] that may modify the state of the component.
    ///
    /// # Errors
    ///
    /// Returns an error if updating the component fails.
    ///
    /// # Returns
    ///
    /// Returns an [`Option<Action>`] to be processed, or `None` if no action is produced.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Renders the component on the screen. (REQUIRED)
    ///
    /// This method draws the component within the specified area using the provided frame.
    /// Override this method to implement custom rendering logic for your component.
    ///
    /// # Arguments
    ///
    /// * `f` - The [`Frame`] used for rendering.
    /// * `area` - The [`Rect`] representing the area in which the component should be drawn.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
