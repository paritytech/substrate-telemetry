use crate::ws_client;
use tokio::process::Command;
use tokio::io::BufReader;
use tokio::io::{ AsyncRead, AsyncBufReadExt };
use tokio::time::Duration;
use anyhow::{ anyhow, Context };

pub struct StartProcesses {
    /// Optional command to run to start a shard (instead of `telemetry_shard`).
    /// The `--listen` argument will be appended here and shouldn't be provided.
    pub shard_command: Option<Command>,
    /// Optional command to run to start a telemetry core process (instead of `telemetry_core`).
    /// The `--listen` argument will be appended here and shouldn't be provided.
    pub telemetry_command: Option<Command>,
    /// How many connections should we establish to each shard? We'll start
    /// up a shard for each entry here.
    pub num_shard_connections: Vec<usize>,
    /// How many connections should we establish to the telemetry_core feed?
    pub num_feed_connections: usize,
}

pub struct ConnectToExisting {
    /// URI to shard /submit endpoint
    pub shards: Vec<ConnectionDetails>,
    /// URI to core /feed endpoint
    pub feed: ConnectionDetails,
}

pub struct ConnectionDetails {
    pub uri: http::Uri,
    pub num_connections: usize
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Can't establsih connection: {0}")]
    ConnectionError(#[from] ws_client::ConnectError),
    #[error("Can't establsih connection: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Can't establsih connection: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not obtain port for process: {0}")]
    ErrorObtainingPort(anyhow::Error)
}

pub struct Connection {
    /// Connections to each of the shard submit URIs
    pub shard_connections: Vec<Vec<(ws_client::Sender, ws_client::Receiver)>>,
    /// Connections to the telemetry feed URI
    pub feed_connections: Vec<(ws_client::Sender, ws_client::Receiver)>
}

impl Connection {
    /// Start telemetry_core and telemetry_shard processes and establish connections to them.
    pub async fn start_processes(opts: StartProcesses) -> Result<Connection, Error> {

        let mut core_cmd = opts.telemetry_command
            .unwrap_or(Command::new("telemetry_core"))
            .arg("--listen")
            .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
            .arg("--log")
            .arg("info")
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        // Find out the port that this is running on
        let core_port = get_port(core_cmd.stdout.take().expect("core stdout"))
            .await
            .map_err(|e| Error::ErrorObtainingPort(e))?;

        let mut shard_cmd = opts.shard_command.unwrap_or(Command::new("telemetry_shard"));
        shard_cmd
            .arg("--listen")
            .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
            .arg("--log")
            .arg("info")
            .arg("--core")
            .arg(format!("127.0.0.1:{}", core_port))
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped());

        // Start shards and find out the ports that they are running on
        let mut shard_ports: Vec<u16> = vec![];
        for _ in 0..opts.num_shard_connections.len() {
            let mut shard_process = shard_cmd.spawn()?;
            let shard_port = get_port(shard_process.stdout.take().expect("shard stdout"))
                .await
                .map_err(|e| Error::ErrorObtainingPort(e))?;

            shard_ports.push(shard_port);
        }

        // now that we've started the processes, establish connections to them:
        let shard_uris: Vec<http::Uri> = shard_ports
            .into_iter()
            .map(|port| format!("http://127.0.0.1:{}/submit", port).parse().expect("valid submit URI"))
            .collect();

        let feed_uri = format!("http://127.0.0.1:{}/feed", core_port)
            .parse()
            .expect("valid feed URI");

        ConnectToExisting {
            feed: ConnectionDetails {
                uri: feed_uri,
                num_connections: opts.num_feed_connections
            },
            shards: opts.num_shard_connections
                .into_iter()
                .zip(shard_uris)
                .map(|(n, uri)| ConnectionDetails {
                    uri,
                    num_connections: n
                })
                .collect()
        };

        todo!();
    }

    /// Establshes the requested connections to existing processes.
    pub async fn connect_to_existing(opts: ConnectToExisting) -> Result<Connection, Error> {
        let shard_details = opts.shards;

        // connect to shards in the background:
        let shard_groups_fut = tokio::spawn(async move {
            let mut shard_results = vec![];
            for details in &shard_details {
                shard_results.push(connect_to_uri(&details.uri, details.num_connections).await);
            }
            let result_shard_groups: Result<Vec<_>,_> = shard_results.into_iter().collect();
            result_shard_groups
        });

        // In the meantime, connect feeds:
        let feed_connections = connect_to_uri(&opts.feed.uri, opts.feed.num_connections).await?;

        // Now feeds are done, wait until shards also connected (this will have been progressing anyway):
        let shard_connections = shard_groups_fut.await??;

        Ok(Connection {
            shard_connections,
            feed_connections,
        })
    }
}

async fn get_port<R: AsyncRead + Unpin>(reader: R) -> Result<u16, anyhow::Error> {
    let reader = BufReader::new(reader);
    let mut reader_lines = reader.lines();

    loop {
        let line = tokio::time::timeout(
            Duration::from_secs(1),
            reader_lines.next_line()
        ).await;

        let line = match line {
            // timeout expired; couldn't get port:
            Err(_) => return Err(anyhow!("Timeout expired waiting to discover port")),
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

        return port_str.parse().with_context(|| "Could not parse output to port");
    }
}

async fn connect_to_uri(uri: &http::Uri, num_connections: usize) -> Result<Vec<(ws_client::Sender, ws_client::Receiver)>, ws_client::ConnectError> {
    let connect_futs = (0..num_connections).map(|_| ws_client::connect(uri));
    let sockets: Result<Vec<_>,_> = futures::future::join_all(connect_futs).await.into_iter().collect();
    sockets
}