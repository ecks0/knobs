mod group;
mod parser;

use std::io::Write as _;
use std::iter::once;

use clap::ErrorKind as ClapErrorKind;
use futures::stream::{self, StreamExt as _, TryStreamExt as _};
use tokio::io::{stderr, stdout, AsyncWriteExt as _, BufWriter};

use crate::cli::group::{Group, Groups};
pub(crate) use crate::cli::parser::{I915Driver, NvmlDriver, Parser};
use crate::util::convert::*;
use crate::util::env::var_name;
use crate::{Cpu, Drm, Error, Nvml, Rapl, Result, I915};

pub(crate) const ARGV0: &str = "knobs";

const QUIET: &str = "quiet";
const SHOW_CPU: &str = "show-cpu";
const SHOW_DRM: &str = "show-drm";
const SHOW_RAPL: &str = "show-rapl";

const QUIET_SHORT: char = 'q';

const GROUP_SEP: &str = "--";

fn config_logging() {
    let env = env_logger::Env::default()
        .filter_or(var_name("LOG"), "error")
        .write_style_or(var_name("LOG_STYLE"), "never");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{:>20}] {}",
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
    pub(crate) help: Option<String>,
    pub(crate) help_long: Option<String>,
    pub(crate) requires: Option<Vec<&'static str>>,
    pub(crate) conflicts: Option<Vec<&'static str>>,
    pub(crate) raw: Option<bool>,
}

fn table_args() -> Vec<Arg> {
    vec![
        Arg {
            name: QUIET.into(),
            long: QUIET.into(),
            short: QUIET_SHORT.into(),
            help: "Do not print tables".to_string().into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_CPU.into(),
            long: SHOW_CPU.into(),
            help: "Print cpu tables".to_string().into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_RAPL.into(),
            long: SHOW_RAPL.into(),
            help: "Print rapl table".to_string().into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_DRM.into(),
            long: SHOW_DRM.into(),
            help: "Print drm tables".to_string().into(),
            ..Default::default()
        },
    ]
}

fn group_arg() -> Vec<Arg> {
    vec![Arg {
        // This argument exists for the sake of --help. It is not parsed by the cli.
        name: "ARGS".into(),
        raw: true.into(),
        help: "Additional option groups".to_string().into(),
        help_long: "Additional option groups delimited by --".to_string().into(),
        ..Default::default()
    }]
}

fn args() -> impl Iterator<Item = Arg> {
    Cpu::args()
        .into_iter()
        .chain(Rapl::args())
        .chain(I915::args())
        .chain(Nvml::args())
        .chain(table_args())
        .chain(group_arg())
}

fn argvs(argv: &[String]) -> Vec<Vec<&str>> {
    log::trace!("cli parse argvs start");
    let r = argv
        .split(|arg| arg == GROUP_SEP)
        .map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            let argv: Vec<_> = once(ARGV0).chain(argv).collect();
            argv
        })
        .collect();
    log::trace!("cli parse argvs done");
    r
}

async fn groups(args: Vec<Arg>, argvs: Vec<Vec<&str>>) -> Result<Groups> {
    log::trace!("cli parse groups start");
    let groups = argvs.into_iter().map(|argv| (&args, argv));
    let groups: Vec<_> = stream::iter(groups)
        .enumerate()
        .map(Ok)
        .and_then(|(i, (args, argv))| async move {
            async {
                let parser = Parser::new(args, &argv)?;
                Group::try_from_ref(&parser).await
            }
            .await
            .map_err(|e| Error::parse_group(e, i + 1))
        })
        .try_collect()
        .await?;
    let groups = Groups::from_iter(groups);
    log::trace!("cli parse groups done");
    Ok(groups)
}

async fn parse(argv: impl IntoIterator<Item = String>) -> Result<(Parser, Groups)> {
    log::trace!("cli parse start");
    let args: Vec<_> = args().collect();
    let argv: Vec<_> = argv.into_iter().skip(1).collect();
    let argvs = argvs(&argv);
    let parser = Parser::new(&args, &argvs[0]).map_err(|e| Error::parse_group(e, 1))?;
    let groups = groups(args, argvs).await?;
    let r = (parser, groups);
    log::trace!("cli parse done");
    Ok(r)
}

async fn tabulate(parser: &Parser, groups: &Groups) -> Result<()> {
    log::trace!("cli tabulate start");
    let show_cpu = parser.flag(SHOW_CPU).is_some();
    let show_rapl = parser.flag(SHOW_RAPL).is_some();
    let show_drm = parser.flag(SHOW_DRM).is_some();
    let has_show_flags = show_cpu || show_rapl || show_drm;
    let has_cpu_vals = groups.has_cpu_values();
    let has_rapl_vals = groups.has_rapl_values();
    let has_i915_vals = groups.has_i915_values();
    let has_nvml_vals = groups.has_nvml_values();
    let has_drm_vals = has_i915_vals || has_nvml_vals;
    let has_vals = has_cpu_vals || has_rapl_vals || has_drm_vals;
    log::trace!("cli tabulate spawn");
    let mut tabulators = vec![];
    if (!has_vals && !has_show_flags) || (has_cpu_vals && !has_show_flags) || show_cpu {
        tabulators.extend(Cpu::tabulate().await);
    }
    if (!has_vals && !has_show_flags) || (has_rapl_vals && !has_show_flags) || show_rapl {
        tabulators.extend(Rapl::tabulate().await);
    }
    if (!has_vals && !has_show_flags) || (has_drm_vals && !has_show_flags) || show_drm {
        tabulators.extend(Drm::tabulate().await);
    }
    let mut stdout = BufWriter::with_capacity(4 * 1024, stdout());
    let mut ok = false;
    log::trace!("cli tabulate print");
    for tabulator in tabulators {
        if let Some(table) = tabulator.await.expect("cli tabulate future") {
            if ok {
                stdout.write_all("\n".as_bytes()).await.unwrap();
            } else {
                ok = true;
            }
            stdout.write_all(table.as_bytes()).await.unwrap();
            log::trace!("cli tabulate print table");
        }
    }
    if ok {
        stdout.flush().await.unwrap();
    } else {
        let mut stderr = stderr();
        stderr.write_all("No supported devices were found\n".as_bytes()).await.unwrap();
        stderr.flush().await.unwrap();
    }
    log::trace!("cli tabulate done");
    Ok(())
}

pub async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    config_logging();
    log::trace!("cli try_run_with_args start");
    let (parser, groups) = parse(argv).await?;
    log::trace!("cli try_run_with_args apply");
    groups.apply().await?;
    if parser.flag(QUIET).is_none() {
        log::trace!("cli try_run_with_args tabulate");
        tabulate(&parser, &groups).await?;
    }
    log::trace!("cli try_run_with_args done");
    Ok(())
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
