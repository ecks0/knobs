use std::path::PathBuf;

use tokio::fs::symlink;
use tokio::io::{stderr, stdout, AsyncWriteExt as _};

use crate::{applet, Error, Result};

pub(super) async fn run(dir: Option<String>) -> Result<()> {
    log::trace!("install run start");
    let argv0 = std::env::current_exe().expect("argv[0] absolute path");
    let dir: PathBuf = if let Some(dir) = dir {
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
    for bin in bins {
        if let Err(e) = symlink(&argv0, &bin).await {
            ok = false;
            let msg = format!("{}: {}\n", bin.display(), e);
            stderr.write_all(msg.as_bytes()).await.unwrap();
        } else {
            let msg = format!("{}\n", bin.display());
            stdout.write_all(msg.as_bytes()).await.unwrap();
        }
    }
    stdout.flush().await.unwrap();
    stderr.flush().await.unwrap();
    log::trace!("install run done");
    if ok { Ok(()) } else { Err(Error::Install) }
}
