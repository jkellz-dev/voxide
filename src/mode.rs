use serde::{Deserialize, Serialize};

/// Represents the different modes of the application.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    /// The default mode, typically representing the home screen.
    #[default]
    Home,
    /// The search mode, used for searching functionality.
    Search,
}
