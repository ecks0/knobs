mod app;
mod parse;
mod sampler;

use tokio::io::{stdout, AsyncWriteExt as _};
use zysfs::tokio::Read as ZysfsRead;

use crate::profile::{Error as ProfileError, Profile};
use crate::{logging, Chain, Error, Result, NAME};

const ARG_QUIET: &str = "quiet";

const ARG_SHOW_CPU: &str = "show-cpu";
const ARG_SHOW_DRM: &str = "show-drm";
#[cfg(feature = "nvml")]
const ARG_SHOW_NVML: &str = "show-nvml";
const ARG_SHOW_PSTATE: &str = "show-pstate";
const ARG_SHOW_RAPL: &str = "show-rapl";

const ARG_CPU: &str = "cpu";
const ARG_CPU_ON: &str = "cpu-on";
const ARG_CPU_ON_EACH: &str = "cpu-on-each";

const ARG_CPUFREQ_GOV: &str = "cpufreq-gov";
const ARG_CPUFREQ_MIN: &str = "cpufreq-min";
const ARG_CPUFREQ_MAX: &str = "cpufreq-max";

const ARG_DRM_I915: &str = "drm-i915";
const ARG_DRM_I915_MIN: &str = "drm-i915-min";
const ARG_DRM_I915_MAX: &str = "drm-i915-max";
const ARG_DRM_I915_BOOST: &str = "drm-i915-boost";

#[cfg(feature = "nvml")]
const ARG_NVML: &str = "nvml";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_MIN: &str = "nvml-gpu-min";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_MAX: &str = "nvml-gpu-max";
#[cfg(feature = "nvml")]
const ARG_NVML_GPU_RESET: &str = "nvml-gpu-reset";
#[cfg(feature = "nvml")]
const ARG_NVML_POWER_LIMIT: &str = "nvml-power-limit";

const ARG_PSTATE_EPB: &str = "pstate-epb";
const ARG_PSTATE_EPP: &str = "pstate-epp";

const ARG_RAPL_PACKAGE: &str = "rapl-package";
const ARG_RAPL_ZONE: &str = "rapl-zone";
const ARG_RAPL_LONG_LIMIT: &str = "rapl-long-limit";
const ARG_RAPL_LONG_WINDOW: &str = "rapl-long-window";
const ARG_RAPL_SHORT_LIMIT: &str = "rapl-short-limit";
const ARG_RAPL_SHORT_WINDOW: &str = "rapl-short-window";

const ARG_PROFILE: &str = "PROFILE";

const ARG_CHAIN: &str = "CHAIN";

const AFTER_HELP: &str = r#"            BOOL   0, 1, true, false
             IDS   comma-delimited sequence of integers and/or integer ranges
           HERTZ*  mhz when unspecified: hz/h - khz/k - mhz/m - ghz/g - thz/t
            SECS   ms when unspecified: ns/n - us/u - ms/m - s
         TOGGLES   sequence of 0 (off), 1 (on), or _ (skip), where position denotes id
           WATTS*  mw when unspecified: uw/u - mw/m - w - kw/k

        * Floating point values may be given for these units.

    All supported values are shown by default unless the --show-* or --quiet flags are used.

    All flags may be expressed as env vars. For example:

        --show-cpu              → KNOBS_SHOW_CPU=1
        --cpu 1,3-5             → KNOBS_CPU=1,3-5
        --drm-i915-boost 1.2ghz → KNOBS_DRM_I915_BOOST=1.2ghz

    The KNOBS_LOG env var may be set to trace, debug, info, warn, or error (default).
"#;

// Command-line interface.
#[derive(Clone, Debug)]
pub struct Cli {
    quiet: Option<()>,
    show_cpu: Option<()>,
    show_drm: Option<()>,
    #[cfg(feature = "nvml")]
    show_nvml: Option<()>,
    show_pstate: Option<()>,
    show_rapl: Option<()>,
    profile: Option<Profile>,
    chains: Vec<Chain>,
}

impl Cli {
    // Create a new instance for the given argv.
    pub async fn new(argv: &[String]) -> Result<Self> {
        log::debug!("Profile config paths: {:#?}", Profile::paths().await);
        let p = parse::Parser::new(argv)?;
        let mut chains = vec![];
        let profile = if let Some(pr) = p.str(ARG_PROFILE) {
            let pr = Profile::new(pr).await?;
            let mut c = pr.read().await?;
            if c.has_values() {
                c.resolve().await;
                chains.push(c);
            }
            Some(pr)
        } else {
            None
        };
        let mut c = Chain::try_from(&p)?;
        if c.has_values() {
            c.resolve().await;
            chains.push(c);
        }
        let s = Self {
            quiet: p.flag(ARG_QUIET),
            show_cpu: p.flag(ARG_SHOW_CPU),
            show_drm: p.flag(ARG_SHOW_DRM),
            #[cfg(feature = "nvml")]
            show_nvml: p.flag(ARG_SHOW_NVML),
            show_pstate: p.flag(ARG_SHOW_PSTATE),
            show_rapl: p.flag(ARG_SHOW_RAPL),
            profile,
            chains,
        };
        Ok(s)
    }

    // Return true if --show-* args are present.
    #[allow(clippy::let_and_return)]
    fn has_show_args(&self) -> bool {
        let b = self.show_cpu.is_some()
            || self.show_drm.is_some()
            || self.show_pstate.is_some()
            || self.show_rapl.is_some();
        #[cfg(feature = "nvml")]
        let b = b || self.show_nvml.is_some();
        b
    }

    // Print values tables.
    async fn print_values(&self, samplers: &sampler::Samplers) -> Result<()> {
        use crate::format::FormatValues as _;
        let mut buf = Vec::with_capacity(3000);
        let show_all = !self.has_show_args();
        if show_all || self.show_cpu.is_some() {
            if let Some(cpu) = zysfs::cpu::Cpu::read(()).await {
                if let Some(cpufreq) = zysfs::cpufreq::Cpufreq::read(()).await {
                    (cpu, cpufreq).format_values(&mut buf, ()).await?;
                }
            }
        }
        if show_all || self.show_pstate.is_some() {
            if let Some(intel_pstate) = zysfs::intel_pstate::IntelPstate::read(()).await {
                intel_pstate.format_values(&mut buf, ()).await?;
            }
        }
        if show_all || self.show_rapl.is_some() {
            if let Some(intel_rapl) = zysfs::intel_rapl::IntelRapl::read(()).await {
                intel_rapl.format_values(&mut buf, samplers.clone().into_samplers()).await?;
            }
        }
        if show_all || self.show_drm.is_some() {
            if let Some(drm) = zysfs::drm::Drm::read(()).await {
                drm.format_values(&mut buf, ()).await?;
            }
        }
        #[cfg(feature = "nvml")]
        if show_all || self.show_nvml.is_some() {
            nvml_facade::Nvml.format_values(&mut buf, ()).await?;
        }
        let s = String::from_utf8_lossy(&buf);
        let mut stdout = stdout();
        stdout.write_all(s[..s.len() - 1].as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
        Ok(())
    }

    // Command-line interface app logic.
    pub async fn run(&self) -> Result<()> {
        let mut samplers = sampler::Samplers::start(self).await;
        for chain in &self.chains {
            chain.apply().await;
        }
        if let Some(p) = self.profile.as_ref() {
            let r = p.set_recent().await;
            if r.is_err() {
                samplers.stop().await;
                r?;
            }
        }
        if self.quiet.is_some() {
            return Ok(());
        } // samplers do not start when quiet
        samplers.wait().await;
        let r = self.print_values(&samplers).await;
        samplers.stop().await;
        r?;
        Ok(())
    }
}

// Cli application.
#[derive(Clone, Debug)]
pub struct App;

impl App {
    // Run app with args.
    pub async fn run_with_args(args: &[String]) {
        logging::configure().await;
        match Cli::new(args).await {
            Ok(cli) => match cli.run().await {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    log::error!("Error: {}", e);
                    std::process::exit(1);
                },
            },
            Err(e) => match &e {
                Error::Clap(e) => {
                    if let clap::ErrorKind::HelpDisplayed = e.kind {
                        let mut s = stdout();
                        s.write_all(e.message.as_bytes()).await.unwrap();
                        s.write_all("\n".as_bytes()).await.unwrap();
                        s.flush().await.unwrap();
                        std::process::exit(0);
                    } else {
                        log::error!("{}", e);
                        std::process::exit(1);
                    }
                },
                Error::Profile(ProfileError::StateCorrupt { path }) => {
                    use tokio::fs::remove_file;
                    log::error!("Error: profile state file corrupted, removing {}", path);
                    match remove_file(path).await {
                        Ok(()) => std::process::exit(1),
                        Err(e) => {
                            log::error!("Error: {}", e);
                            std::process::exit(1);
                        },
                    }
                },
                _ => {
                    log::error!("Error: {}", e);
                    std::process::exit(1);
                },
            },
        }
    }

    // Run app.
    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();
        Self::run_with_args(&args).await
    }
}
