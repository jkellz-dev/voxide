//! Defines the `Action` enum, which represents all possible actions/events that can occur in the application.
//!
//! This includes UI events (tick, render, resize), user commands (quit, help, search),
//! and domain-specific actions (play/stop station, update mode, etc).
//!
//! The `Action` enum is central to the application's event-driven architecture.
use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

use crate::{
    mode::Mode as AppMode,
    models::{RadioStation, SearchParam},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    /// Represents a periodic tick event, typically used for UI updates.
    Tick,
    /// Represents a render event to redraw the UI.
    Render,
    /// Represents a terminal resize event with new width and height.
    Resize(u16, u16),
    /// Represents suspension of the application (e.g., for shelling out).
    Suspend,
    /// Represents resuming the application after suspension.
    Resume,
    /// Represents a request to quit the application.
    Quit,
    /// Represents a request to refresh the application's state or data.
    Refresh,
    /// Represents an error event with an associated message.
    Error(String),
    /// Represents a request to show help information.
    Help,
    /// Toggles the visibility of the help UI.
    ToggleShowHelp,
    /// Moves to the next item in a list or menu.
    NextItem,
    /// Moves to the previous item in a list or menu.
    PreviousItem,
    /// Switches the application to normal mode.
    EnterNormal,
    /// Switches the application to insert mode.
    EnterInsert,
    /// Switches the application to processing mode.
    EnterProcessing,
    /// Exits the processing mode.
    ExitProcessing,
    /// Represents an update event for the application's state.
    Update,
    /// Initiates a search with the given search parameters.
    Search(Vec<SearchParam>),
    /// Indicates that a list of radio stations has been found.
    StationsFound(Vec<RadioStation>),
    /// Requests playback of the currently selected radio station.
    PlaySelectedStation,
    /// Requests stopping playback of the current radio station.
    StopPlayingStation,
    /// Updates the application's mode.
    Mode(AppMode),
    /// Switches the application to search mode.
    SearchMode,
    /// Switches the application to home mode.
    HomeMode,
    /// Increases the audio volume.
    IncreaseVolume,
    /// Decreases the audio volume.
    DecreaseVolume,
}
