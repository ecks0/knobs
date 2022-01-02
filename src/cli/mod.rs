mod parser;

pub(crate) use parser::{I915Driver, NvmlDriver, Parser};
use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt as _};

use crate::util::convert::*;
use crate::{Cpu, Drm, Error, Nvml, Groups, Rapl, Result, I915};

const QUIET: &str = "quiet";
const SHOW_CPU: &str = "show-cpu";
const SHOW_DRM: &str = "show-drm";
const SHOW_RAPL: &str = "show-rapl";
const ARGS: &str = "ARGS";

const QUIET_SHORT: &str = "q";

#[derive(Debug, Default)]
pub(crate) struct Arg {
    pub(crate) name: Option<&'static str>,
    pub(crate) long: Option<&'static str>,
    pub(crate) short: Option<&'static str>,
    pub(crate) value_name: Option<&'static str>,
    pub(crate) help: Option<String>,
    pub(crate) help_long: Option<String>,
    pub(crate) requires: Option<Vec<&'static str>>,
    pub(crate) conflicts: Option<Vec<&'static str>>,
    pub(crate) raw: Option<bool>,
}

fn args_before() -> Vec<Arg> {
    vec![
        Arg {
            name: QUIET.into(),
            long: QUIET.into(),
            short: QUIET_SHORT.into(),
            help: "Do not print tables".to_string().into(),
            conflicts: vec![SHOW_CPU, SHOW_RAPL, SHOW_DRM].into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_CPU.into(),
            long: SHOW_CPU.into(),
            help: "Show cpu tables".to_string().into(),
            conflicts: vec![QUIET].into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_RAPL.into(),
            long: SHOW_RAPL.into(),
            help: "Show rapl table".to_string().into(),
            conflicts: vec![QUIET].into(),
            ..Default::default()
        },
        Arg {
            name: SHOW_DRM.into(),
            long: SHOW_DRM.into(),
            help: "Show drm tables".to_string().into(),
            conflicts: vec![QUIET].into(),
            ..Default::default()
        },
    ]
}

fn args_after() -> Vec<Arg> {
    vec![
        Arg {
            name: ARGS.into(),
            raw: true.into(),
            ..Default::default()
        },
    ]
}

fn args() -> impl Iterator<Item = Arg> {
    args_before()
        .into_iter()
        .chain(Cpu::args())
        .chain(Rapl::args())
        .chain(I915::args())
        .chain(Nvml::args())
        .chain(args_after())
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

async fn tabulate(parser: &Parser<'_>, groups: &Groups) -> Result<()> {
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
        if let Some(v) = Cpu::tabulate().await {
            tables.push(v);
        }
    }
    if (!has_vals && !has_show_flags) || (has_rapl_vals && !has_show_flags) || show_rapl {
        if let Some(v) = Rapl::tabulate().await {
            tables.push(v);
        }
    }
    if (!has_vals && !has_show_flags) || (has_drm_vals && !has_show_flags) || show_drm {
        if let Some(v) = Drm::tabulate().await {
            tables.push(v);
        }
        if let Some(v) = I915::tabulate().await {
            tables.push(v);
        }
        if let Some(v) = Nvml::tabulate().await {
            tables.push(v);
        }
    }
    if tables.is_empty() {
        eprint("No supported devices were found", true).await;
    } else {
        let tables = tables.join("\n");
        print(&tables, false).await;
    }
    Ok(())
}

async fn try_run_with_args(argv: impl IntoIterator<Item = String>) -> Result<()> {
    let argv: Vec<_> = argv.into_iter().collect();
    let args: Vec<_> = args().collect();
    let parser = Parser::new(&args, &argv)?;
    let quiet = parser.flag(QUIET).is_some();
    let groups = Groups::try_from_ref(&parser).await?;
    groups.apply().await?;
    if !quiet {
        tabulate(&parser, &groups).await?;
    }
    Ok(())
}

async fn run_with_args(argv: impl IntoIterator<Item = String>) {
    if let Err(e) = try_run_with_args(argv).await {
        match e {
            Error::Clap(e) => {
                if let clap::ErrorKind::HelpDisplayed = e.kind {
                    print(&e.to_string(), false).await;
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
