use std::path::PathBuf;

use async_trait::async_trait;
use futures::future::{self, FutureExt as _};
use tokio::fs::symlink;
use tokio::io::{stdout, AsyncWriteExt as _};

use crate::applet::{self, Applet, Formatter};
use crate::cli::{Arg, Parser};
use crate::{Error, Result};

const DIR: &str = "dir";

#[derive(Debug, Default)]
pub(crate) struct Install;

#[async_trait]
impl Applet for Install {
    fn name(&self) -> &'static str {
        "install"
    }

    fn bin(&self) -> Option<String> {
        None
    }

    fn about(&self) -> &'static str {
        "Install command symlinks"
    }

    fn args(&self) -> Vec<Arg> {
        vec![Arg {
            name: DIR.into(),
            help: "Override installation directory".into(),
            ..Default::default()
        }]
    }

    async fn run(&mut self, p: Parser<'_>) -> Result<()> {
        log::trace!("install run start");
        let argv0 = std::env::current_exe().expect("argv[0] absolute path");
        let dir: PathBuf = if let Some(dir) = p.str(DIR) {
            dir.into()
        } else {
            argv0.parent().expect("parent directory of argv[0]").into()
        };
        let bins: Vec<_> =
            applet::all().into_iter().filter_map(|a| a.bin().map(|v| dir.join(v))).collect();
        let mut stdout = stdout();
        for bin in bins {
            symlink(&argv0, &bin).await.map_err(|e| Error::Install(e, bin.clone()))?;
            let msg = format!("{}\n", bin.display());
            stdout.write_all(msg.as_bytes()).await.unwrap();
        }
        stdout.flush().await.unwrap();
        log::trace!("install run done");
        Ok(())
    }

    async fn summary(&self) -> Vec<Formatter> {
        vec![future::ready(Some(String::new())).boxed()]
    }

    fn default_summary(&self) -> bool {
        false
    }
}
