mod parser;

use std::collections::HashSet;
use std::iter;

use clap::ErrorKind as ClapErrorKind;
use futures::future::join_all;
use tokio::io::{stderr, stdout, AsyncWriteExt as _, BufWriter};

use crate::applet::{self, Applet, Runner};
pub(crate) use crate::cli::parser::{I915Driver, NvmlDriver, Parser};
use crate::util::env::var_name;
use crate::{Error, Result};

pub(crate) const NAME: &str = "knobs";

const QUIET: &str = "quiet";
const QUIET_SHORT: char = 'q';
const QUIET_HELP: &str = "Do not print tables";

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

fn make_app_args() -> Vec<Arg> {
    vec![Arg {
        name: QUIET.into(),
        long: QUIET.into(),
        short: QUIET_SHORT.into(),
        help: QUIET_HELP.into(),
        ..Default::default()
    }]
}

struct App {
    argv0: String,
    argv: Vec<String>,
    applets: Vec<Box<dyn Applet>>,
    quiet: bool,
    runners: Vec<(usize, Runner)>,
    format_subcmds: HashSet<&'static str>,
}

impl App {
    fn new(argv: impl IntoIterator<Item = String>) -> Self {
        log::trace!("app init start");
        let mut argv = argv.into_iter();
        let r = Self {
            argv0: argv
                .next()
                .expect("argv[0]")
                .split('/')
                .last()
                .expect("basename of argv[0]")
                .to_string(),
            argv: argv.collect(),
            applets: applet::all(),
            quiet: false,
            runners: Default::default(),
            format_subcmds: Default::default(),
        };
        log::trace!("app init done");
        r
    }

    async fn run(mut self) -> Result<()> {
        log::trace!("app run start");
        if self.is_binary() {
            self.make_binary_runners().await?;
        } else {
            self.make_subcommand_runners().await?;
        };
        self.join_runners().await?;
        if !self.quiet {
            self.format().await;
        }
        log::trace!("app run done");
        Ok(())
    }

    fn is_binary(&self) -> bool {
        let argv0 = self.argv0.as_str();
        self.applets.iter().any(|a| Some(argv0) == a.binary())
    }

    async fn make_binary_runners(&mut self) -> Result<()> {
        log::trace!("app make binary runners start");
        let applet = self
            .applets
            .iter()
            .find(|a| Some(self.argv0.as_str()) == a.binary())
            .expect("applet for binary");
        let app_args_data = make_app_args();
        let applet_args_data = applet.args();
        let app_args = app_args_data.iter().map(clap::Arg::from);
        let applet_args = applet_args_data.iter().map(clap::Arg::from).chain(app_args);
        let argvs = self.argv.split(|v| "--" == v).map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            iter::once(self.argv0.as_str()).chain(argv)
        });
        for (i, argv) in argvs.enumerate() {
            async {
                let app =
                    make_clap_app(&self.argv0).about(applet.about()).args(applet_args.clone());
                let matches = app.try_get_matches_from(argv)?;
                let parser = Parser::from(&matches);
                if parser.flag(QUIET).is_some() {
                    self.quiet = true;
                }
                let runner = applet.run(parser).await?;
                self.runners.push((i, runner));
                Ok(())
            }
            .await
            .map_err(|e| Error::group(e, i + 1))?;
        }
        if !self.quiet {
            self.format_subcmds.insert(applet.subcommand());
        }
        log::trace!("app make binary runners done");
        Ok(())
    }

    async fn make_subcommand_runners(&mut self) -> Result<()> {
        log::trace!("app make subcommand runners start");
        let app_args_data = make_app_args();
        let applet_args_data: Vec<_> =
            self.applets.iter().map(|a| (a.subcommand(), a.about(), a.args())).collect();
        let app_args = app_args_data.iter().map(clap::Arg::from);
        let applet_args = applet_args_data
            .iter()
            .map(|(n, a, args)| (*n, *a, args.iter().map(clap::Arg::from).collect::<Vec<_>>()));
        let argvs = self.argv.split(|v| "--" == v).map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            iter::once(self.argv0.as_str()).chain(argv)
        });
        for (i, argv) in argvs.enumerate() {
            async {
                let matches = applet_args
                    .clone()
                    .fold(
                        make_clap_app(&self.argv0).args(app_args.clone()),
                        |clap_app, (name, about, args)| {
                            let subcmd = make_clap_app(name).about(about).args(args);
                            clap_app.subcommand(subcmd)
                        },
                    )
                    .try_get_matches_from(argv)?;
                if Parser::from(&matches).flag(QUIET).is_some() {
                    self.quiet = true;
                    self.format_subcmds.clear();
                }
                if let Some((subcmd, subcmd_matches)) = matches.subcommand() {
                    let applet = self
                        .applets
                        .iter()
                        .find(|a| subcmd == a.subcommand())
                        .expect("applet for subcommand");
                    let parser = Parser::from(subcmd_matches);
                    let runner = applet.run(parser).await?;
                    self.runners.push((i, runner));
                    if !self.quiet {
                        self.format_subcmds.insert(applet.subcommand());
                    }
                }
                Ok(())
            }
            .await
            .map_err(|e| Error::group(e, i + 1))?;
        }
        log::trace!("app make subcommand runners done");
        Ok(())
    }

    async fn join_runners(&mut self) -> Result<()> {
        log::trace!("app join runners start");
        for (i, runner) in self.runners.drain(..) {
            runner.await.map_err(|e| Error::group(e, i + 1))?;
        }
        log::trace!("app join runners done");
        Ok(())
    }

    async fn format(&mut self) {
        log::trace!("app format start");
        let formatters = self.applets.iter().filter_map(|a| {
            if self.format_subcmds.is_empty() || self.format_subcmds.contains(a.subcommand()) {
                Some(a.format())
            } else {
                None
            }
        });
        let formatters = join_all(formatters).await.into_iter().flatten().flatten();
        let outputs: Vec<_> = join_all(formatters).await.into_iter().flatten().collect();
        if !outputs.is_empty() {
            let mut stdout = BufWriter::with_capacity(4 * 1024, stdout());
            let mut ok = false;
            for output in outputs {
                if ok {
                    stdout.write_all("\n".as_bytes()).await.unwrap();
                } else {
                    ok = true;
                }
                stdout.write_all(output.as_bytes()).await.unwrap();
            }
            stdout.flush().await.unwrap();
        }
        log::trace!("app format done");
    }
}

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

pub async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    config_logging();
    App::new(argv).run().await
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
