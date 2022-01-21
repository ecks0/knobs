mod parser;

use std::io::Write as _;

use clap::ErrorKind as ClapErrorKind;
use futures::future::join_all;
use tokio::io::{stderr, stdout, AsyncWriteExt as _, BufWriter};

use crate::applet::{self, Applet, Formatter};
pub(crate) use crate::cli::parser::{I915Driver, NvmlDriver, Parser};
use crate::util::env::var_name;
use crate::{Error, Result};

pub(crate) const NAME: &str = "knobs";

fn config_logging() {
    let env = env_logger::Env::default()
        .filter_or(var_name("LOG"), "error")
        .write_style_or(var_name("LOG_STYLE"), "never");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{:>32}] {}",
                chrono::Local::now().format("%H:%M:%S%.6f"),
                record.level().to_string().chars().next().unwrap_or('-'),
                record.target(),
                record.args()
            )
        })
        .init()
}

#[derive(Debug, Default)]
pub(crate) struct Arg {
    pub(crate) name: Option<&'static str>,
    pub(crate) long: Option<&'static str>,
    pub(crate) short: Option<char>,
    pub(crate) value_name: Option<&'static str>,
    pub(crate) help: Option<&'static str>,
    pub(crate) help_long: Option<String>,
    pub(crate) required: Option<bool>,
    pub(crate) requires: Option<Vec<&'static str>>,
    pub(crate) conflicts: Option<Vec<&'static str>>,
    pub(crate) raw: Option<bool>,
}

impl<'a> From<&'a Arg> for clap::Arg<'a> {
    fn from(v: &'a Arg) -> Self {
        let name = v.name.expect("Cli argument name is missing");
        let mut a = clap::Arg::new(name);
        if let Some(long) = v.long {
            a = a.long(long);
        }
        if let Some(short) = v.short {
            a = a.short(short);
        }
        if let Some(value_name) = v.value_name {
            a = a.takes_value(true).value_name(value_name);
        }
        if let Some(help) = v.help {
            a = a.help(help);
        }
        if let Some(help_long) = &v.help_long {
            a = a.long_help(help_long.as_str());
        }
        if let Some(required) = v.required {
            a = a.required(required);
        }
        if let Some(requires) = &v.requires {
            for required in requires {
                a = a.requires(required);
            }
        }
        if let Some(conflicts) = &v.conflicts {
            for conflicted in conflicts {
                a = a.conflicts_with(conflicted);
            }
        }
        if let Some(raw) = v.raw {
            a = a.raw(raw);
        }
        a
    }
}

fn make_clap_app<'a, S>(name: S) -> clap::App<'a>
where
    S: Into<String>,
{
    clap::App::new(name)
        .color(clap::ColorChoice::Never)
        .setting(clap::AppSettings::DeriveDisplayOrder)
        .setting(clap::AppSettings::DisableHelpSubcommand)
        .version(clap::crate_version!())
}

async fn format(formatters: Vec<Formatter>) {
    let mut stdout = BufWriter::with_capacity(4 * 1024, stdout());
    let mut ok = false;
    for output in join_all(formatters).await.into_iter().flatten() {
        if ok {
            stdout.write_all("\n".as_bytes()).await.unwrap();
        } else {
            ok = true;
        }
        stdout.write_all(output.as_bytes()).await.unwrap();
    }
    if ok {
        stdout.flush().await.unwrap();
    }
}

async fn run_subcommand(argv: Vec<String>, applets: Vec<Box<dyn Applet>>) -> Result<()> {
    let applet_args: Vec<_> = applets.iter().map(|a| (a, a.args())).collect();
    let matches = applet_args
        .iter()
        .fold(make_clap_app(NAME), |clap_app, (applet, args)| {
            let subcmd_args = args.iter().map(clap::Arg::from);
            let subcmd = make_clap_app(applet.name()).args(subcmd_args).about(applet.about());
            clap_app.subcommand(subcmd)
        })
        .try_get_matches_from(argv)?;
    drop(applet_args);
    let formatters = match matches.subcommand() {
        Some((subcmd, subcmd_matches)) => {
            let mut applet = applets.into_iter().find(|a| a.name() == subcmd).unwrap();
            let parser = Parser::from(subcmd_matches);
            applet.run(parser).await?;
            drop(matches);
            applet.summary().await
        },
        None => {
            drop(matches);
            let default_summaries = applets.into_iter().map(|applet| async move {
                if applet.default_summary() { Some(applet.summary().await) } else { None }
            });
            join_all(default_summaries).await.into_iter().flatten().flatten().collect()
        },
    };
    format(formatters).await;
    Ok(())
}

async fn run_binary(bin: &str, argv: Vec<String>, applets: Vec<Box<dyn Applet>>) -> Result<()> {
    let mut applet =
        applets.into_iter().find(|a| a.bin().map(|v| bin == v).unwrap_or(false)).unwrap();
    let applet_args = applet.args();
    let clap_args: Vec<_> = applet_args.iter().map(clap::Arg::from).collect();
    let matches = make_clap_app(bin).args(clap_args).try_get_matches_from(argv)?;
    drop(applet_args);
    let parser = Parser::from(&matches);
    applet.run(parser).await?;
    drop(matches);
    let formatters = applet.summary().await;
    drop(applet);
    format(formatters).await;
    Ok(())
}

fn argv0(argv: &[String]) -> String {
    argv.iter().next().and_then(|s| s.as_str().split('/').last()).unwrap_or(NAME).to_string()
}

pub async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    config_logging();
    let argv: Vec<_> = argv.into_iter().collect();
    let argv0 = argv0(&argv);
    let applets = applet::all();
    if applets.iter().any(|a| a.bin().map(|v| argv0 == v).unwrap_or(false)) {
        run_binary(&argv0, argv, applets).await
    } else {
        run_subcommand(argv, applets).await
    }
}

pub async fn run_with_args(argv: impl IntoIterator<Item = String>) {
    if let Err(e) = try_run_with_args(argv).await {
        match e {
            Error::Clap(e) => {
                if matches!(
                    e.kind,
                    ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion
                ) {
                    let mut stdout = stdout();
                    stdout.write_all(e.to_string().as_bytes()).await.unwrap();
                    stdout.flush().await.unwrap();
                    std::process::exit(0);
                } else {
                    let mut stderr = stderr();
                    stderr.write_all(e.to_string().as_bytes()).await.unwrap();
                    stderr.write_all("\n".as_bytes()).await.unwrap();
                    stderr.flush().await.unwrap();
                    std::process::exit(1);
                }
            },
            _ => {
                let mut stderr = stderr();
                stderr.write_all(e.to_string().as_bytes()).await.unwrap();
                stderr.write_all("\n".as_bytes()).await.unwrap();
                stderr.flush().await.unwrap();
                std::process::exit(2);
            },
        }
    }
    std::process::exit(0);
}

pub async fn run() {
    let args = std::env::args();
    run_with_args(args).await
}
