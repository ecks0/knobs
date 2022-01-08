use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt as _};

async fn write<W>(w: &mut W, s: &str, nl: bool, flush: bool)
where
    W: AsyncWrite + Send + Unpin,
{
    let _ = w.write_all(s.as_bytes()).await;
    if nl {
        let _ = w.write_all("\n".as_bytes()).await;
    }
    if flush {
        let _ = w.flush().await;
    }
}

pub(crate) async fn print(s: &str, nl: bool, flush: bool) {
    let mut w = stdout();
    write(&mut w, s, nl, flush).await
}

pub(crate) async fn eprint(s: &str, nl: bool, flush: bool) {
    let mut w = stderr();
    write(&mut w, s, nl, flush).await
}
