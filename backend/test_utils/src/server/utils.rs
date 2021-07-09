use crate::ws_client;
use tokio::io::BufReader;
use tokio::io::{ AsyncRead, AsyncWrite, AsyncBufReadExt };
use tokio::time::Duration;
use anyhow::{ anyhow, Context };

/// Reads from the stdout of the shard/core process to extract the port that was assigned to it,
/// with the side benefit that we'll wait for it to start listening before returning. We do this
/// because we want to allow the kernel to assign ports and so don't specify a port as an arg.
pub async fn get_port<R: AsyncRead + Unpin>(reader: R) -> Result<u16, anyhow::Error> {
    let reader = BufReader::new(reader);
    let mut reader_lines = reader.lines();

    loop {
        let line = tokio::time::timeout(
            // This has to accomodate pauses during compilation if the cmd is "cargo run --":
            Duration::from_secs(30),
            reader_lines.next_line()
        ).await;

        let line = match line {
            // timeout expired; couldn't get port:
            Err(e) => return Err(anyhow!("Timeout expired waiting to discover port: {}", e)),
            // Something went wrong reading line; bail:
            Ok(Err(e)) => return Err(anyhow!("Could not read line from stdout: {}", e)),
            // No more output; process ended? bail:
            Ok(Ok(None)) => return Err(anyhow!("No more output from stdout; has the process ended?")),
            // All OK, and a line is given back; phew!
            Ok(Ok(Some(line))) => line
        };

        let (_, port_str) = match line.rsplit_once("listening on http://127.0.0.1:") {
            Some(m) => m,
            None => continue
        };

        return port_str
            .trim()
            .parse()
            .with_context(|| format!("Could not parse output to port: {}", port_str));
    }
}

/// Establish multiple connections to a URI and return them all.
pub async fn connect_multiple_to_uri(uri: &http::Uri, num_connections: usize) -> Result<Vec<(ws_client::Sender, ws_client::Receiver)>, ws_client::ConnectError> {
    let connect_futs = (0..num_connections)
        .map(|_| ws_client::connect(uri));
    let sockets: Result<Vec<_>,_> = futures::future::join_all(connect_futs)
        .await
        .into_iter()
        .collect();
    sockets
}

/// Drain output from a reader to stdout. After acquiring port details from spawned processes,
/// they expect their stdout to be continue to be consumed, and so we do this here.
pub fn drain<R, W>(mut reader: R, mut writer: W)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static
{
    tokio::spawn(async move {
        let _ = tokio::io::copy(&mut reader, &mut writer).await;
    });
}