mod summary;

use async_trait::async_trait;

use crate::applet::{Applet, Formatter};
use crate::cli::{Arg, Parser};
use crate::Result;

#[derive(Debug, Default)]
pub(crate) struct Drm;

#[async_trait]
impl Applet for Drm {
    fn name(&self) -> &'static str {
        "drm"
    }

    fn bin(&self) -> Option<&'static str> {
        Some("kdrm")
    }

    fn about(&self) -> &'static str {
        "View drm values"
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }

    async fn run(&mut self, _: Parser<'_>) -> Result<()> {
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        summary::summary().await
    }

    fn default_summary(&self) -> bool {
        true
    }
}
