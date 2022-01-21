mod cpu;
mod drm;
mod i915;
mod install;
mod nvml;
mod rapl;

use std::pin::Pin;

use async_trait::async_trait;
pub(crate) use cpu::Cpu;
pub(crate) use drm::Drm;
use futures::future::Future;
pub(crate) use i915::I915;
pub(crate) use install::Install;
pub(crate) use nvml::Nvml;
pub(crate) use rapl::{ConstraintIds as RaplConstraintIds, Rapl};

use crate::cli::{Arg, Parser};
use crate::Result;

pub(crate) type Formatter = Pin<Box<dyn Future<Output = Option<String>> + Send>>;

pub(crate) fn all() -> Vec<Box<dyn Applet>> {
    vec![
        Box::new(Cpu::default()),
        Box::new(Rapl::default()),
        Box::new(Drm::default()),
        Box::new(I915::default()),
        Box::new(Nvml::default()),
        Box::new(Install::default()),
    ]
}

#[async_trait]
pub(crate) trait Applet {
    fn name(&self) -> &'static str;

    fn bin(&self) -> Option<&'static str>;

    fn about(&self) -> &'static str;

    fn args(&self) -> Vec<Arg>;

    async fn run(&mut self, parser: Parser<'_>) -> Result<()>;

    async fn summary(&self) -> Vec<Formatter>;

    fn default_summary(&self) -> bool;
}
