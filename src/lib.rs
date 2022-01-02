mod cli;
mod cpu;
mod i915;
mod nvml;
mod group;
mod rapl;
mod util;

pub use clap::Error as ClapError;
pub(crate) use cpu::Cpu;
pub(crate) use i915::I915;
pub(crate) use nvml::Nvml;
pub(crate) use group::{Groups, Group};
pub(crate) use rapl::Rapl;
pub use syx::Error as SyxError;
pub use tokio::io::Error as IoError;

pub use crate::cli::run;

const NAME: &str = "knobs";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Clap(#[from] ClapError),

    #[error("{0}")]
    TableIo(IoError),

    #[error(transparent)]
    Syx(#[from] SyxError),

    #[error("--{flag}: {error}")]
    ParseFlag { flag: String, error: String },

    #[error("{0}")]
    ParseValue(String),
}

impl Error {
    fn parse_flag(flag: impl Into<String>, error: impl ToString) -> Self {
        let flag = flag.into();
        let error = error.to_string();
        Self::ParseFlag { flag, error }
    }

    fn parse_value(message: impl ToString) -> Self {
        let message = message.to_string();
        Self::ParseValue(message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
