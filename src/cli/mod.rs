mod parser;

pub(crate) use parser::{I915Driver, NvmlDriver, Parser};
use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt as _};

use crate::util::convert::*;
use crate::{Cpu, Error, Nvml, Profiles, Rapl, Result, I915};

const QUIET: &str = "quiet";
const SHOW_CPU: &str = "show-cpu";
const SHOW_I915: &str = "show-i915";
const SHOW_NVML: &str = "show-nvml";
const SHOW_RAPL: &str = "show-rapl";

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

fn table_args() -> Vec<Arg> {
    vec![
        Arg {
            name: QUIET,
            long: QUIET.into(),
            short: QUIET_SHORT.into(),
            value_name: None,
            help: "Do not print tables".to_string().into(),
            help_long: None,
            requires: None,
            conflicts: vec![SHOW_CPU, SHOW_RAPL, SHOW_I915, SHOW_NVML].into(),
        },
        Arg {
            name: SHOW_CPU,
            long: SHOW_CPU.into(),
            short: None,
            value_name: None,
            help: "Show cpu table".to_string().into(),
            help_long: None,
            requires: None,
            conflicts: vec![QUIET].into(),
        },
        Arg {
            name: SHOW_RAPL,
            long: SHOW_RAPL.into(),
            short: None,
            value_name: None,
            help: "Show rapl table".to_string().into(),
            help_long: None,
            requires: None,
            conflicts: vec![QUIET].into(),
        },
        Arg {
            name: SHOW_I915,
            long: SHOW_I915.into(),
            short: None,
            value_name: None,
            help: "Show i915 table".to_string().into(),
            help_long: None,
            requires: None,
            conflicts: vec![QUIET].into(),
        },
        Arg {
            name: SHOW_NVML,
            long: SHOW_NVML.into(),
            short: None,
            value_name: None,
            help: "Show nvml table".to_string().into(),
            help_long: None,
            requires: None,
            conflicts: vec![QUIET].into(),
        },
    ]
}

fn args() -> impl Iterator<Item = Arg> {
    table_args()
        .into_iter()
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

async fn tabulate(parser: &Parser<'_>, profiles: &Profiles) -> Result<()> {
    let show_cpu = parser.flag(SHOW_CPU).is_some();
    let show_rapl = parser.flag(SHOW_RAPL).is_some();
    let show_i915 = parser.flag(SHOW_I915).is_some();
    let show_nvml = parser.flag(SHOW_NVML).is_some();
    let has_show_flags = show_cpu || show_rapl || show_i915 || show_nvml;
    let has_cpu_vals = profiles.has_cpu_values();
    let has_rapl_vals = profiles.has_rapl_values();
    let has_i915_vals = profiles.has_i915_values();
    let has_nvml_vals = profiles.has_nvml_values();
    let has_values = has_cpu_vals || has_rapl_vals || has_i915_vals || has_nvml_vals;
    let mut tables = vec![];
    if (!has_values && !has_show_flags) || (has_cpu_vals && !has_show_flags) || show_cpu {
        if let Some(v) = Cpu::tabulate().await {
            tables.push(v);
        }
    }
    if (!has_values && !has_show_flags) || (has_rapl_vals && !has_show_flags) || show_rapl {
        if let Some(v) = Rapl::tabulate().await {
            tables.push(v);
        }
    }
    if (!has_values && !has_show_flags) || (has_i915_vals && !has_show_flags) || show_i915 {
        if let Some(v) = I915::tabulate().await {
            tables.push(v);
        }
    }
    if (!has_values && !has_show_flags) || (has_nvml_vals && !has_show_flags) || show_nvml {
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
    let profiles = Profiles::try_from_ref(&parser).await?;
    profiles.apply().await?;
    if !quiet {
        tabulate(&parser, &profiles).await?;
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
