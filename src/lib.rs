mod applet;
mod cli;
mod util;

use std::fmt::Display;

pub use clap::Error as ClapError;
pub use syx::Error as SyxError;
pub use tokio::io::Error as IoError;

pub use crate::cli::{run, run_with_args, try_run_with_args};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error("failed to create one or more symlinks")]
    Install,

    #[error("error: {0}")]
    Syx(#[from] SyxError),

    #[error("error: --{flag}: {error}")]
    ParseFlag { error: String, flag: String },

    #[error("error: {0}")]
    ParseValue(String),
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
}

pub type Result<T> = std::result::Result<T, Error>;
