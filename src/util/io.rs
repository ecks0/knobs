use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt as _};

async fn write<W>(w: &mut W, msg: &str, nl: bool)
where
    W: AsyncWrite + Send + Unpin,
{
    let _ = w.write_all(msg.as_bytes()).await;
    if nl {
        let _ = w.write_all("\n".as_bytes()).await;
    }
    let _ = w.flush().await;
}

pub(crate) async fn print(msg: &str, nl: bool) {
    let mut w = stdout();
    write(&mut w, msg, nl).await
}

pub(crate) async fn eprint(msg: &str, nl: bool) {
    let mut w = stderr();
    write(&mut w, msg, nl).await
}
