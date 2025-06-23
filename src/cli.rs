use std::path::PathBuf;

use clap::Parser;

use crate::utils::version;

/// Command-line interface options for the application.
///
/// This struct defines the arguments that can be passed to the application via the command line.
#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second.
    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Tick rate, i.e. number of ticks per second",
        default_value_t = 1.0
    )]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second.
    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Frame rate, i.e. number of frames per second",
        default_value_t = 4.0
    )]
    pub frame_rate: f64,
}
