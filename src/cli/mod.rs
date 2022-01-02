mod parser;

use std::iter::once;

pub(crate) use parser::{I915Driver, NvmlDriver, Parser};
use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt as _};

use crate::util::convert::*;
use crate::{Cpu, Error, Nvml, Profiles, Rapl, Result, I915};

const QUIET: &str = "quiet";
const QUIET_SHORT: &str = "q";

#[derive(Debug)]
pub(crate) struct Arg {
    pub(crate) name: &'static str,
    pub(crate) long: Option<&'static str>,
    pub(crate) short: Option<&'static str>,
    pub(crate) value_name: Option<&'static str>,
    pub(crate) help: Option<String>,
    pub(crate) help_long: Option<String>,
    pub(crate) requires: Option<Vec<&'static str>>,
    pub(crate) conflicts: Option<Vec<&'static str>>,
}

fn quiet() -> Arg {
    Arg {
        name: QUIET,
        long: QUIET.into(),
        short: QUIET_SHORT.into(),
        value_name: None,
        help: "Do not print tables".to_string().into(),
        help_long: None,
        requires: None,
        conflicts: None,
    }
}

fn args() -> impl Iterator<Item = Arg> {
    once(quiet())
        .chain(Cpu::args())
        .chain(Rapl::args())
        .chain(I915::args())
        .chain(Nvml::args())
}

async fn write<W>(w: &mut W, msg: &str, nl: bool)
where
    W: AsyncWrite + Send + Unpin,
{
    let _ = w.write_all(msg.as_bytes()).await;
    if nl {
        let _ = w.write_all("\n".as_bytes()).await;
    }
    let _ = w.flush().await;
}

async fn print(msg: &str, nl: bool) {
    let mut w = stdout();
    write(&mut w, msg, nl).await
}

async fn eprint(msg: &str, nl: bool) {
    let mut w = stderr();
    write(&mut w, msg, nl).await
}

async fn tabulate() -> Result<()> {
    let tables = [
        Cpu::tabulate().await,
        Rapl::tabulate().await,
        I915::tabulate().await,
        Nvml::tabulate().await,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !tables.is_empty() {
        let tables = tables.join("\n");
        print(&tables, false).await;
    }
    Ok(())
}

async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    let argv: Vec<_> = argv.into_iter().collect();
    let args: Vec<_> = args().collect();
    let parser = Parser::new(&args, &argv)?;
    let quiet = parser.flag(QUIET);
    let profiles = Profiles::try_from_ref(&parser).await?;
    profiles.apply().await?;
    if quiet.is_none() {
        tabulate().await?;
    }
    Ok(())
}

async fn run_with_args(argv: impl IntoIterator<Item = String>) {
    if let Err(e) = try_run_with_args(argv).await {
        match e {
            Error::Clap(e) => {
                if let clap::ErrorKind::HelpDisplayed = e.kind {
                    print(&e.message, true).await;
                    std::process::exit(0);
                } else {
                    eprint(&e.to_string(), true).await;
                    std::process::exit(1);
                }
            },
            _ => {
                eprint(&format!("Error: {}", e), true).await;
                std::process::exit(2);
            },
        }
    }
}

pub async fn run() {
    let args = std::env::args();
    run_with_args(args).await
}
