pub(crate) mod cpu;
pub(crate) mod drm;
pub(crate) mod i915;
pub(crate) mod install;
pub(crate) mod nvml;
pub(crate) mod rapl;

use std::pin::Pin;

use async_trait::async_trait;
use futures::future::Future;

use crate::cli::{Arg, Parser};
use crate::Result;

pub(crate) type Formatter = Pin<Box<dyn Future<Output = Option<String>> + Send>>;

pub(crate) fn all() -> Vec<Box<dyn Applet>> {
    vec![
        Box::new(cpu::Cpu::default()),
        Box::new(rapl::Rapl::default()),
        Box::new(drm::Drm::default()),
        Box::new(i915::I915::default()),
        Box::new(nvml::Nvml::default()),
        Box::new(install::Install::default()),
    ]
}

#[async_trait]
pub(crate) trait Applet {
    fn name(&self) -> &'static str;

    fn bin(&self) -> Option<String> {
        Some(format!("k{}", self.name()))
    }

    fn about(&self) -> &'static str;

    fn args(&self) -> Vec<Arg>;

    async fn run(&mut self, parser: Parser<'_>) -> Result<()>;

    async fn summary(&self) -> Vec<Formatter>;

    fn default_summary(&self) -> bool;
}
