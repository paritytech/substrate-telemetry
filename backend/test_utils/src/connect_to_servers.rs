use crate::ws_client;
use tokio::process::{ self, Command };
use tokio::io::BufReader;
use tokio::io::{ AsyncRead, AsyncBufReadExt };
use tokio::time::Duration;
use anyhow::{ anyhow, Context };

pub struct StartProcessOpts {
    /// Optional command to run to start a shard (instead of `telemetry_shard`).
    /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
    pub shard_command: Option<Command>,
    /// How many shards should we start?
    pub num_shards: usize,
    /// Optional command to run to start a telemetry core process (instead of `telemetry_core`).
    /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
    pub core_command: Option<Command>
}

pub struct ConnectToExistingOpts {
    /// Details for connections to `telemetry_shard` /submit endpoints
    pub shard_uris: Vec<http::Uri>,
    /// Details for connections to `telemetry_core` /feed endpoints
    pub feed_uri: http::Uri,
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
    ErrorObtainingPort(anyhow::Error),
    #[error("Whoops; attempt to kill a process we didn't start (and so have no handle to)")]
    CannotKillNoHandle
}

/// This provides back connections (or groups of connections) that are
/// hooked up to the running processes and ready to send/receive messages.
pub struct Server {
    /// Shard processes that we can connect to
    pub shards: Vec<Process>,
    /// Core process that we can connect to
    pub core: Process,
}

impl Server {
    /// Start telemetry_core and telemetry_shard processes and establish connections to them.
    pub async fn start_processes(opts: StartProcessOpts) -> Result<Server, Error> {

        let mut core_cmd = opts.core_command
            .unwrap_or(Command::new("telemetry_core"))
            .arg("--listen")
            .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
            .arg("--log")
            .arg("info")
            .kill_on_drop(true)
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
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped());

        // Start shards and find out the ports that they are running on
        let mut shard_handle_and_ports: Vec<(process::Child, u16)> = vec![];
        for _ in 0..opts.num_shards {
            let mut shard_process = shard_cmd.spawn()?;
            let shard_port = get_port(shard_process.stdout.take().expect("shard stdout"))
                .await
                .map_err(|e| Error::ErrorObtainingPort(e))?;

                shard_handle_and_ports.push((shard_process, shard_port));
        }

        // now that we've started the processes, establish connections to them:
        let shard_handle_and_uris: Vec<(process::Child, http::Uri)> = shard_handle_and_ports
            .into_iter()
            .map(|(h,port)| (h,format!("http://127.0.0.1:{}/submit", port).parse().expect("valid submit URI")))
            .collect();

        let feed_uri = format!("http://127.0.0.1:{}/feed", core_port)
            .parse()
            .expect("valid feed URI");

        Ok(Server {
            shards: shard_handle_and_uris
                .into_iter()
                .map(|(handle, uri)| Process {
                    handle: Some(handle),
                    uri,
                })
                .collect(),
            core: Process {
                handle: Some(core_cmd),
                uri: feed_uri,
            }
        })
    }

    /// Establshes the requested connections to existing processes.
    pub fn connect_to_existing(opts: ConnectToExistingOpts) -> Server {
        Server {
            shards: opts.shard_uris
                .into_iter()
                .map(|uri| Process { uri, handle: None })
                .collect(),
            core: Process { uri: opts.feed_uri, handle: None }
        }
    }
}

/// This represents a running process that we can connect to, which
/// may be either a `telemetry_shard` or `telemetry_core`.
pub struct Process {
    /// If we started the processes ourselves, we'll have a handle to
    /// them which we can use to kill them. Else, we may not.
    handle: Option<process::Child>,
    /// The URI that we can use to connect to the process socket.
    uri: http::Uri
}

impl Process {
    /// Establish a connection to the process
    pub async fn connect(&self) -> Result<(ws_client::Sender, ws_client::Receiver), Error> {
        ws_client::connect(&self.uri)
            .await
            .map_err(|e| e.into())
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple(&self, num_connections: usize) -> Result<Vec<(ws_client::Sender, ws_client::Receiver)>, Error> {
        connect_multiple_to_uri(&self.uri, num_connections)
            .await
            .map_err(|e| e.into())
    }

    /// Kill the process and wait for this to complete
    pub async fn kill(self) -> Result<(), Error> {
        match self.handle {
            Some(mut handle) => Ok(handle.kill().await?),
            None => Err(Error::CannotKillNoHandle)
        }
    }
}

/// Reads from the stdout of the shard/core process to extract the port that was assigned to it,
/// with the side benefit that we'll wait for it to start listening before returning. We do this
/// because we want to allow the kernel to assign ports and so don't specify a port as an arg.
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

async fn connect_multiple_to_uri(uri: &http::Uri, num_connections: usize) -> Result<Vec<(ws_client::Sender, ws_client::Receiver)>, ws_client::ConnectError> {
    let connect_futs = (0..num_connections)
        .map(|_| ws_client::connect(uri));
    let sockets: Result<Vec<_>,_> = futures::future::join_all(connect_futs)
        .await
        .into_iter()
        .collect();
    sockets
}