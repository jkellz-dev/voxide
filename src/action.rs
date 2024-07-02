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
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,
    ToggleShowHelp,
    NextItem,
    PreviousItem,
    EnterNormal,
    EnterInsert,
    EnterProcessing,
    ExitProcessing,
    Update,
    Search(Vec<SearchParam>),
    StationsFound(Vec<RadioStation>),
    PlaySelectedStation,
    StopPlayingStation,
    Mode(AppMode),
    SearchMode,
    HomeMode,
}
