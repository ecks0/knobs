use std::path::PathBuf;

use tokio::fs::{remove_file, symlink};
use tokio::io::{stderr, stdout, AsyncWriteExt as _};

use crate::{applet, Error, Result};

pub(super) async fn run(values: super::Values) -> Result<()> {
    log::trace!("install run start");
    let argv0 = std::env::current_exe().expect("absolute path of argv[0]");
    let dir: PathBuf = if let Some(dir) = values.dir {
        dir.into()
    } else {
        argv0.parent().expect("parent directory of argv[0]").into()
    };
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut ok = true;
    let bins: Vec<_> = applet::all()
        .into_iter()
        .filter_map(|a| a.binary().map(|v| dir.join(v)))
        .collect();
    if values.uninstall.is_none() {
        for bin in bins {
            if let Err(e) = symlink(&argv0, &bin).await {
                ok = false;
                let msg = format!("{}: {}\n", bin.display(), e);
                stderr.write_all(msg.as_bytes()).await.unwrap();
            } else {
                let msg = format!("{}\n", bin.display());
                stdout.write_all(msg.as_bytes()).await.unwrap();
            };
        }
    } else {
        for bin in bins {
            if let Err(e) = remove_file(&bin).await {
                ok = false;
                let msg = format!("{}: {}\n", bin.display(), e);
                stderr.write_all(msg.as_bytes()).await.unwrap();
            } else {
                let msg = format!("{}\n", bin.display());
                stdout.write_all(msg.as_bytes()).await.unwrap();
            };
        }
    }
    stdout.flush().await.unwrap();
    stderr.flush().await.unwrap();
    log::trace!("install run done");
    if ok {
        Ok(())
    } else {
        Err(if values.uninstall.is_none() { Error::Install } else { Error::Uninstall })
    }
}
