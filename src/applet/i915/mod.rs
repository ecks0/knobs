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
    min: Option<Frequency>,
    max: Option<Frequency>,
    boost: Option<Frequency>,
}

#[derive(Debug, Default)]
pub(crate) struct I915;

#[async_trait]
impl Applet for I915 {
    fn binary(&self) -> Option<&'static str> {
        Some("k915")
    }

    fn subcommand(&self) -> &'static str {
        "915"
    }

    fn about(&self) -> &'static str {
        "View or set i915 values"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&self, p: Parser<'_>) -> Result<Runner> {
        let values = Values::from_parser(p).await?;
        let r = run::run(values).boxed();
        Ok(r)
    }

    async fn format(&self) -> Vec<Formatter> {
        format::format().await
    }
}
