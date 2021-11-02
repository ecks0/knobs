use zysfs::types::{self as sysfs, tokio::Feature as _};
use std::time::Duration;
use tokio::time::sleep;
use crate::cli::{counter, Cli};
use crate::data::{RaplSampler, RaplSamplers};

#[derive(Clone, Debug)]
pub struct Samplers {
    samplers: Option<RaplSamplers>,
}

impl Samplers {
    // Minimum run time required to get give useful data.
    const RUNTIME: Duration = Duration::from_millis(400);

    // Sleep time between samples, per sampler/zone.
    const INTERVAL: Duration = Duration::from_millis(100);

    /// Maximum number of samples, per sampler/zone.
    const COUNT: usize = 11;

    pub async fn new(cli: &Cli) -> Self {
        let samplers =
            if cli.quiet.is_none() &&
                (!cli.has_show_args() || cli.show_rapl.is_some()) &&
                sysfs::intel_rapl::IntelRapl::present().await
            {
                if let Some(s) = RaplSampler::all(Self::INTERVAL, Self::COUNT).await {
                    let mut s = RaplSamplers::from(s);
                    s.start().await;
                    Some(s)
                } else { None }
            } else { None };
        Self { samplers }
    }

    pub async fn stop(&mut self) {
        if let Some(s) = self.samplers.as_mut() {
            s.stop().await;
        }
    }

    pub async fn wait(&self, begin: Duration) {
        if let Some(s) = self.samplers.as_ref() {
            if s.working().await {
                let runtime = counter::delta().await - begin;
                if runtime < Self::RUNTIME {
                    sleep(Self::RUNTIME - runtime).await;
                }
            }
        }
    }

    pub fn into_option(self) -> Option<RaplSamplers> { self.samplers }
}
