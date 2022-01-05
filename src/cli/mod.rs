mod group;
mod parser;

use std::iter::once;

use futures::stream::{self, StreamExt as _, TryStreamExt as _};

use crate::cli::group::{Group, Groups};
pub(crate) use crate::cli::parser::{I915Driver, NvmlDriver, Parser};
use crate::util::convert::*;
use crate::util::io::{eprint, print};
use crate::{Cpu, Drm, Error, Nvml, Rapl, Result, I915};

const ARGV0: &str = "knobs";

const QUIET: &str = "quiet";
const SHOW_CPU: &str = "show-cpu";
const SHOW_DRM: &str = "show-drm";
const SHOW_RAPL: &str = "show-rapl";

const QUIET_SHORT: char = 'q';

const GROUP_SEP: &str = "--";

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
    // This argument exists to inform users about argument
    // groups via -h/--help. It is not parsed by the cli.
    vec![Arg {
        name: "ARGS".into(),
        raw: true.into(),
        help: "Additional option groups".to_string().into(),
        help_long: "Additional option groups delimited by --".to_string().into(),
        ..Default::default()
    }]
}

fn args() -> impl Iterator<Item = Arg> {
    table_args()
        .into_iter()
        .chain(Cpu::args())
        .chain(Rapl::args())
        .chain(I915::args())
        .chain(Nvml::args())
        .chain(group_arg())
}

fn argvs(argv: &[String]) -> Vec<Vec<&str>> {
    argv.split(|arg| arg == GROUP_SEP)
        .map(|argv| {
            let argv = argv.iter().map(|v| v.as_str());
            let argv: Vec<_> = once(ARGV0).chain(argv).collect();
            argv
        })
        .collect()
}

async fn groups(args: Vec<Arg>, argvs: Vec<Vec<&str>>) -> Result<Groups> {
    let groups = argvs.into_iter().map(|argv| (&args, argv));
    let groups: Vec<_> = stream::iter(groups)
        .enumerate()
        .map(Result::Ok)
        .and_then(|(i, (args, argv))| async move {
            let r = async {
                let parser = Parser::new(args, &argv)?;
                let r = Group::try_from_ref(&parser).await?;
                Ok(r)
            }
            .await
            .map_err(|e| Error::parse_group(e, i + 1))?;
            Ok(r)
        })
        .try_collect()
        .await?;
    let groups = Groups::from_iter(groups);
    Ok(groups)
}

async fn parse(argv: impl IntoIterator<Item = String>) -> Result<(Parser, Groups)> {
    let args: Vec<_> = args().collect();
    let argv: Vec<_> = argv.into_iter().skip(1).collect();
    let argvs = argvs(&argv);
    let parser = Parser::new(&args, &argvs[0]).map_err(|e| Error::parse_group(e, 1))?;
    let groups = groups(args, argvs).await?;
    let r = (parser, groups);
    Ok(r)
}

async fn tabulate(parser: &Parser, groups: &Groups) -> Result<()> {
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
    let mut tables = vec![];
    if (!has_vals && !has_show_flags) || (has_cpu_vals && !has_show_flags) || show_cpu {
        tables.extend(Cpu::tabulate().await);
    }
    if (!has_vals && !has_show_flags) || (has_rapl_vals && !has_show_flags) || show_rapl {
        tables.extend(Rapl::tabulate().await);
    }
    if (!has_vals && !has_show_flags) || (has_drm_vals && !has_show_flags) || show_drm {
        tables.extend(Drm::tabulate().await);
        tables.extend(I915::tabulate().await);
        tables.extend(Nvml::tabulate().await);
    }
    if tables.is_empty() {
        eprint("No supported devices were found", true).await;
    } else {
        let tables = tables.join("\n");
        print(&tables, false).await;
    }
    Ok(())
}

pub async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    let (parser, groups) = parse(argv).await?;
    groups.apply().await?;
    if parser.flag(QUIET).is_none() {
        tabulate(&parser, &groups).await?;
    }
    Ok(())
}

pub async fn run_with_args(argv: impl IntoIterator<Item = String>) {
    if let Err(e) = try_run_with_args(argv).await {
        match e {
            Error::Clap(e) => {
                if let clap::ErrorKind::DisplayHelp = e.kind {
                    print(&e.to_string(), false).await;
                    std::process::exit(0);
                } else {
                    eprint(&e.to_string(), true).await;
                    std::process::exit(1);
                }
            },
            _ => {
                eprint(&e.to_string(), true).await;
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
