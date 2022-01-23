mod cpu;
mod i915;
mod install;
mod nvml;
mod rapl;

use std::pin::Pin;

use async_trait::async_trait;
use futures::future::Future;

pub(crate) use crate::applet::cpu::Cpu;
pub(crate) use crate::applet::i915::I915;
pub(crate) use crate::applet::install::Install;
pub(crate) use crate::applet::nvml::Nvml;
pub(crate) use crate::applet::rapl::{ConstraintIds as RaplConstraintIds, Rapl};
use crate::cli::{Arg, Parser};
use crate::Result;

pub(crate) type Runner = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub(crate) type Formatter = Pin<Box<dyn Future<Output = Option<String>> + Send>>;

pub(crate) fn all() -> Vec<Box<dyn Applet>> {
    vec![
        Box::new(Cpu::default()),
        Box::new(Rapl::default()),
        Box::new(I915::default()),
        Box::new(Nvml::default()),
        Box::new(Install::default()),
    ]
}

#[async_trait]
pub(crate) trait Applet {
    fn binary(&self) -> Option<&'static str>;

    fn subcommand(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn args(&self) -> Vec<Arg>;

    async fn run(&self, parser: Parser<'_>) -> Result<Runner>;

    async fn format(&self) -> Vec<Formatter>;
}
