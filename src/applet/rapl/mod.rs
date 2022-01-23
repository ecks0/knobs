mod args;
mod format;
mod run;

use std::time::Duration;

use async_trait::async_trait;
use futures::future::FutureExt as _;
use measurements::Power;

use crate::app::{Arg, Parser};
use crate::applet::{Applet, Formatter, Runner};
use crate::Result;

#[derive(Debug)]
pub(crate) struct ConstraintIds {
    pub(crate) package: u64,
    pub(crate) subzone: Option<u64>,
    pub(crate) constraints: Vec<u64>,
}

#[derive(Debug)]
struct Values {
    constraint_ids: Option<ConstraintIds>,
    limit: Option<Power>,
    window: Option<Duration>,
}

#[derive(Debug, Default)]
pub(crate) struct Rapl;

#[async_trait]
impl Applet for Rapl {
    fn binary(&self) -> Option<&'static str> {
        Some("krapl")
    }

    fn subcommand(&self) -> &'static str {
        "rapl"
    }

    fn about(&self) -> &'static str {
        "View or set rapl values"
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
