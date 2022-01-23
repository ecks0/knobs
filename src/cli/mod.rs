mod parser;

use std::collections::HashSet;
use std::iter;

use clap::ErrorKind as ClapErrorKind;
use futures::future::join_all;
use tokio::io::{stderr, stdout, AsyncWriteExt as _, BufWriter};

use crate::applet::{self, Applet, Formatter};
pub(crate) use crate::cli::parser::{I915Driver, NvmlDriver, Parser};
use crate::util::env::var_name;
use crate::{Error, Result};

pub(crate) const NAME: &str = "knobs";

const QUIET: &str = "quiet";
const QUIET_SHORT: char = 'q';
const QUIET_HELP: &str = "Do not print tables";

fn config_logging() {
    use std::io::Write as _;
    let env = env_logger::Env::default()
        .filter_or(var_name("LOG"), "error")
        .write_style_or(var_name("LOG_STYLE"), "never");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{:>30}] {}",
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
        let name = v.name.expect("cli argument name");
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

fn app_args() -> Vec<Arg> {
    vec![Arg {
        name: QUIET.into(),
        long: QUIET.into(),
        short: QUIET_SHORT.into(),
        help: QUIET_HELP.into(),
        ..Default::default()
    }]
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

async fn run_subcommands<'a>(
    argv0: &'a str,
    argv: &'a [String],
    applets: Vec<Box<dyn Applet>>,
) -> Result<()> {
    let (quiet, subcmds, runners) = async {
        let app_args_data = app_args();
        let applet_args_data: Vec<_> =
            applets.iter().map(|a| (a.subcommand(), a.about(), a.args())).collect();
        let app_args = app_args_data.iter().map(clap::Arg::from);
        let applet_args = applet_args_data
            .iter()
            .map(|(n, a, args)| (*n, *a, args.iter().map(clap::Arg::from).collect::<Vec<_>>()));
        let mut quiet = false;
        let mut subcmds = HashSet::new();
        let mut runners = vec![];
        let argvs = argv.split(|v| "--" == v).map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            iter::once(argv0).chain(argv)
        });
        for (i, argv) in argvs.enumerate() {
            async {
                let matches = applet_args
                    .clone()
                    .fold(
                        make_clap_app(argv0).args(app_args.clone()),
                        |clap_app, (name, about, args)| {
                            let subcmd = make_clap_app(name).about(about).args(args);
                            clap_app.subcommand(subcmd)
                        },
                    )
                    .try_get_matches_from(argv)?;
                if Parser::from(&matches).flag(QUIET).is_some() {
                    quiet = true;
                }
                if let Some((subcmd, subcmd_matches)) = matches.subcommand() {
                    let applet = applets
                        .iter()
                        .find(|a| subcmd == a.subcommand())
                        .expect("applet for subcommand");
                    let parser = Parser::from(subcmd_matches);
                    let runner = applet.run(parser).await?;
                    runners.push((i, runner));
                    subcmds.insert(applet.subcommand());
                }
                Ok(())
            }
            .await
            .map_err(|e| Error::group(e, i))?;
        }
        Result::Ok((quiet, subcmds, runners))
    }
    .await?;
    let formatters = async move {
        let mut formatters = vec![];
        if !quiet {
            for applet in applets {
                if subcmds.is_empty() || subcmds.contains(applet.subcommand()) {
                    formatters.extend(applet.format().await);
                }
            }
        }
        formatters
    }
    .await;
    for (i, runner) in runners {
        runner.await.map_err(|e| Error::group(e, i))?;
    }
    format(formatters).await;
    Ok(())
}

async fn run_binary<'a>(
    argv0: &'a str,
    argv: &'a [String],
    applets: Vec<Box<dyn Applet>>,
) -> Result<()> {
    let applet = applets
        .into_iter()
        .find(|a| Some(argv0) == a.binary())
        .expect("applet for binary");
    let (quiet, runners) = async {
        let app_args_data = app_args();
        let applet_args_data = applet.args();
        let app_args = app_args_data.iter().map(clap::Arg::from);
        let applet_args = applet_args_data.iter().map(clap::Arg::from).chain(app_args);
        let mut quiet = false;
        let mut runners = vec![];
        let argvs = argv.split(|v| "--" == v).map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            iter::once(argv0).chain(argv)
        });
        for (i, argv) in argvs.enumerate() {
            async {
                let app = make_clap_app(argv0).about(applet.about()).args(applet_args.clone());
                let matches = app.try_get_matches_from(argv)?;
                let parser = Parser::from(&matches);
                if parser.flag(QUIET).is_some() {
                    quiet = true;
                }
                let runner = applet.run(parser).await?;
                runners.push((i, runner));
                Ok(())
            }
            .await
            .map_err(|e| Error::group(e, i))?;
        }
        Result::Ok((quiet, runners))
    }
    .await?;
    let formatters = async move {
        if quiet { vec![] } else { applet.format().await }
    }
    .await;
    for (i, runner) in runners {
        runner.await.map_err(|e| Error::group(e, i))?;
    }
    format(formatters).await;
    Ok(())
}

fn argv0(argv: &[String]) -> &str {
    argv.iter()
        .next()
        .and_then(|s| s.as_str().split('/').last())
        .expect("basename of argv[0]")
}

pub async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    config_logging();
    let argv: Vec<_> = argv.into_iter().collect();
    let argv0 = argv0(&argv);
    let applets = applet::all();
    if applets.iter().any(|a| Some(argv0) == a.binary()) {
        run_binary(argv0, &argv[1..], applets).await
    } else {
        run_subcommands(argv0, &argv[1..], applets).await
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
