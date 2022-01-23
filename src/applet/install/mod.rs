mod args;
mod run;

use async_trait::async_trait;
use futures::future::FutureExt as _;

use crate::applet::{Applet, Formatter, Runner};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug)]
struct Values {
    uninstall: Option<()>,
    dir: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Install;

#[async_trait]
impl Applet for Install {
    fn binary(&self) -> Option<&'static str> {
        None
    }

    fn subcommand(&self) -> &'static str {
        "install"
    }

    fn about(&self) -> &'static str {
        "Install utility symlinks"
    }

    fn args(&self) -> Vec<Arg> {
        args::args()
    }

    async fn run(&self, p: Parser<'_>) -> Result<Runner> {
        let values = Values::from_parser(p);
        let r = run::run(values).boxed();
        Ok(r)
    }

    async fn format(&self) -> Option<Vec<Formatter>> {
        None
    }
}
