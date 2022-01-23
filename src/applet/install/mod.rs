mod run;

use async_trait::async_trait;
use futures::future::FutureExt as _;

use crate::applet::{Applet, Formatter, Runner};
use crate::cli::{Arg, Parser};
use crate::Result;

const DIR: &str = "dir";

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
        vec![Arg {
            name: DIR.into(),
            help: "Specify symlink installation directory".into(),
            ..Default::default()
        }]
    }

    async fn run(&self, p: Parser<'_>) -> Result<Runner> {
        let dir = p.string(DIR);
        let r = run::run(dir).boxed();
        Ok(r)
    }

    async fn format(&self) -> Vec<Formatter> {
        vec![]
    }
}
