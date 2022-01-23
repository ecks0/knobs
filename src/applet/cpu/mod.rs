mod args;
mod format;
mod run;

use async_trait::async_trait;
use futures::future::FutureExt as _;
use measurements::Frequency;

use crate::applet::{Applet, Formatter, Runner};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
struct Values {
    ids: Option<Vec<u64>>,
    on: Option<bool>,
    gov: Option<String>,
    min: Option<Frequency>,
    max: Option<Frequency>,
    epb: Option<u64>,
    epp: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Cpu;

#[async_trait]
impl Applet for Cpu {
    fn binary(&self) -> Option<&'static str> {
        Some("kcpu")
    }

    fn subcommand(&self) -> &'static str {
        "cpu"
    }

    fn about(&self) -> &'static str {
        "View or set cpu values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
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
