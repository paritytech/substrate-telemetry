use common::ws_client;
use anyhow::{anyhow, Context};
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite};
use tokio::time::Duration;

/// Reads from the stdout of the shard/core process to extract the port that was assigned to it,
/// with the side benefit that we'll wait for it to start listening before returning. We do this
/// because we want to allow the kernel to assign ports and so don't specify a port as an arg.
pub async fn get_port<R: AsyncRead + Unpin>(reader: R) -> Result<u16, anyhow::Error> {
    // For the new service:
    let new_expected_text = "listening on http://127.0.0.1:";
    // For the older non-sharded actix based service:
    let old_expected_text = "service on 127.0.0.1:";

    let is_text = |s: &str| s.contains(new_expected_text) || s.contains(old_expected_text);
    wait_for_line_containing(reader, is_text, Duration::from_secs(240))
        .await
        .and_then(|line| {
            // The line must match one of our expected strings:
            let (_, port_str) = line
                .rsplit_once(new_expected_text)
                .unwrap_or_else(|| line.rsplit_once(old_expected_text).unwrap());
            // Grab the port after the string:
            port_str
                .trim()
                .parse()
                .with_context(|| format!("Could not parse output to port: {}", port_str))
        })
}

/// Wait for a line of output containing the text given. Also provide a timeout,
/// such that if we don't see a new line of output within the timeout we bail out
/// and return an error.
pub async fn wait_for_line_containing<R: AsyncRead + Unpin, F: Fn(&str) -> bool>(
    reader: R,
    is_match: F,
    max_wait_between_lines: Duration,
) -> Result<String, anyhow::Error> {
    let reader = BufReader::new(reader);
    let mut reader_lines = reader.lines();

    loop {
        let line = tokio::time::timeout(max_wait_between_lines, reader_lines.next_line()).await;

        let line = match line {
            // timeout expired; couldn't get port:
            Err(_) => {
                return Err(anyhow!("Timeout elapsed waiting for text match"))
            }
            // Something went wrong reading line; bail:
            Ok(Err(e)) => {
                return Err(anyhow!("Could not read line from stdout: {}", e))
            },
            // No more output; process ended? bail:
            Ok(Ok(None)) => {
                return Err(anyhow!(
                    "No more output from stdout; has the process ended?"
                ))
            }
            // All OK, and a line is given back; phew!
            Ok(Ok(Some(line))) => line,
        };

        if is_match(&line) {
            return Ok(line);
        }
    }
}

/// Establish multiple connections to a URI and return them all.
pub async fn connect_multiple_to_uri(
    uri: &http::Uri,
    num_connections: usize,
) -> Result<Vec<(ws_client::Sender, ws_client::Receiver)>, ws_client::ConnectError> {
    // Previous versions of this used future::join_all to concurrently establish all of the
    // connections we want. However, trying to do that with more than say ~130 connections on
    // MacOS led to hitting "Connection reset by peer" errors, so let's do it one-at-a-time.
    // (Side note: on Ubuntu the concurrency seemed to be no issue up to at least 10k connections).
    let mut sockets = vec![];
    for _ in 0..num_connections {
        sockets.push(ws_client::connect(uri).await?);
    }
    Ok(sockets)
}

/// Drain output from a reader to stdout. After acquiring port details from spawned processes,
/// they expect their stdout to be continue to be consumed, and so we do this here.
pub fn drain<R, W>(mut reader: R, mut writer: W)
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let _ = tokio::io::copy(&mut reader, &mut writer).await;
    });
}
