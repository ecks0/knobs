mod app;
mod applet;
mod util;

use std::fmt::Display;

pub use clap::Error as ClapError;
pub use syx::Error as SyxError;
pub use tokio::io::Error as IoError;

pub use crate::app::run;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error("failed to create one or more symlinks")]
    Install,

    #[error("failed to remove one or more symlinks")]
    Uninstall,

    #[error("{0}")]
    Syx(#[from] SyxError),

    #[error("--{flag}: {error}")]
    ParseFlag { error: String, flag: String },

    #[error("{0}")]
    ParseValue(String),

    #[error("argument group {1}: {0}")]
    Group(String, usize),
}

impl Error {
    fn parse_flag(error: Self, flag: impl Display) -> Self {
        let error = error.to_string();
        let flag = flag.to_string();
        Self::ParseFlag { error, flag }
    }

    fn parse_value(message: impl Display) -> Self {
        let message = message.to_string();
        Self::ParseValue(message)
    }

    fn group(error: Self, group: usize) -> Self {
        if let Error::Clap(err) = &error {
            if matches!(
                err.kind,
                clap::ErrorKind::DisplayHelp
                    | clap::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) {
                return error;
            }
        }
        let error = error.to_string();
        Self::Group(error, group)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
