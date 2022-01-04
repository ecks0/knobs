mod cli;
mod cpu;
mod drm;
mod i915;
mod nvml;
mod rapl;
mod util;

use std::fmt::Display;

pub use clap::Error as ClapError;
pub use syx::Error as SyxError;
pub use tokio::io::Error as IoError;

pub use crate::cli::{run, run_with_args, try_run_with_args};
pub(crate) use crate::cpu::Cpu;
pub(crate) use crate::drm::Drm;
pub(crate) use crate::i915::I915;
pub(crate) use crate::nvml::Nvml;
pub(crate) use crate::rapl::Rapl;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("group {group}:\n{error}")]
    ApplyGroup { error: String, group: usize },

    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error(transparent)]
    Syx(#[from] SyxError),

    #[error("--{flag}: {error}")]
    ParseFlag { error: String, flag: String },

    #[error("group {group}: {error}")]
    ParseGroup { error: String, group: usize },

    #[error("{0}")]
    ParseValue(String),
}

impl Error {
    fn apply_group(error: Self, group: usize) -> Self {
        let error = error.to_string();
        Self::ParseGroup { error, group }
    }

    fn parse_flag(error: Self, flag: impl Display) -> Self {
        let error = error.to_string();
        let flag = flag.to_string();
        Self::ParseFlag { error, flag }
    }

    fn parse_group(error: Self, group: usize) -> Self {
        let error = error.to_string();
        Self::ParseGroup { error, group }
    }

    fn parse_value(message: impl Display) -> Self {
        let message = message.to_string();
        Self::ParseValue(message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
