mod args;
mod format;
mod run;

use async_trait::async_trait;
use futures::future::FutureExt as _;
use measurements::{Frequency, Power};

use crate::applet::{Applet, Formatter, Runner};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
struct Values {
    cards: Option<Vec<u64>>,
    gpu_min: Option<Frequency>,
    gpu_max: Option<Frequency>,
    gpu_reset: Option<()>,
    power: Option<Power>,
    power_reset: Option<()>,
}

#[derive(Debug, Default)]
pub(crate) struct Nvml;

#[async_trait]
impl Applet for Nvml {
    fn binary(&self) -> Option<&'static str> {
        Some("knvml")
    }

    fn subcommand(&self) -> &'static str {
        "nvml"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    fn about(&self) -> &'static str {
        "View or set nvml values"
    }

    async fn run(&self, p: Parser<'_>) -> Result<Runner> {
        let values = Values::from_parser(p).await?;
        let r = run::run(values).boxed();
        Ok(r)
    }

    async fn format(&self) -> Option<Vec<Formatter>> {
        Some(format::format().await)
    }
}
