use super::{channels, utils};
use common::ws_client;
use common::{id_type, DenseMap};
use std::ffi::OsString;
use std::marker::PhantomData;
use tokio::process::{self, Command as TokioCommand};

id_type! {
    /// The ID of a running process. Cannot be constructed externally.
    pub struct ProcessId(usize);
}

pub enum StartOpts {
    /// Start a single core process that is expected
    /// to have both `/feed` and `/submit` endpoints
    SingleProcess {
        /// Command to run to start the process.
        /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
        command: Command,
    },
    /// Start a core process with a `/feed` andpoint as well as (optionally)
    /// multiple shard processes with `/submit` endpoints.
    ShardAndCore {
        /// Command to run to start a shard.
        /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
        shard_command: Command,
        /// Command to run to start a telemetry core process.
        /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
        core_command: Command,
    },
    /// Connect to existing process(es).
    ConnectToExisting {
        /// Where are the processes that we can `/submit` things to?
        /// Eg: `vec![127.0.0.1:12345, 127.0.0.1:9091]`
        submit_hosts: Vec<String>,
        /// Where is the process that we can subscribe to the `/feed` of?
        /// Eg: `127.0.0.1:3000`
        feed_host: String,
    }
}

/// This represents a telemetry server. It can be in different modes
/// depending on how it was started, but the interface is similar in every case
/// so that tests are somewhat compatible with multiple configurations.
pub enum Server {
    SingleProcessMode {
        /// A virtual shard that we can hand out.
        virtual_shard: ShardProcess,
        /// Core process that we can connect to.
        core: CoreProcess
    },
    ShardAndCoreMode {
        /// Command to run to start a new shard.
        shard_command: Command,
        /// Shard processes that we can connect to.
        shards: DenseMap<ProcessId, ShardProcess>,
        /// Core process that we can connect to.
        core: CoreProcess,
    },
    ConnectToExistingMode {
        /// The hosts that we can connect to to submit things.
        submit_hosts: Vec<String>,
        /// Which host do we use next (we'll cycle around them
        /// as shards are "added").
        next_submit_host_idx: usize,
        /// Shard processes that we can connect to.
        shards: DenseMap<ProcessId, ShardProcess>,
        /// Core process that we can connect to.
        core: CoreProcess,
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Can't establsih connection: {0}")]
    ConnectionError(#[from] ws_client::ConnectError),
    #[error("Can't establsih connection: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Can't establsih connection: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not obtain port for process as the line we waited for in log output didn't show up: {0}")]
    ErrorObtainingPort(anyhow::Error),
    #[error("Whoops; attempt to kill a process we didn't start (and so have no handle to)")]
    CannotKillNoHandle,
    #[error(
        "Can't add a shard: command not provided, or we are not in charge of spawning processes"
    )]
    CannotAddShard,
    #[error("The URI provided was invalid: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri)
}

impl Server {
    pub fn get_core(&self) -> &CoreProcess {
        match self {
            Server::SingleProcessMode { core, .. } => core,
            Server::ShardAndCoreMode { core, ..} => core,
            Server::ConnectToExistingMode { core, .. } => core
        }
    }

    pub fn get_shard(&self, id: ProcessId) -> Option<&ShardProcess> {
        match self {
            Server::SingleProcessMode { virtual_shard, .. } => Some(virtual_shard),
            Server::ShardAndCoreMode { shards, ..} => shards.get(id),
            Server::ConnectToExistingMode { shards, .. } => shards.get(id)
        }
    }

    pub async fn kill_shard(&mut self, id: ProcessId) -> bool {
        let shard = match self {
            // Can't remove the pretend shard:
            Server::SingleProcessMode { .. } => return false,
            Server::ShardAndCoreMode { shards, ..} => shards.remove(id),
            Server::ConnectToExistingMode { shards, .. } => shards.remove(id)
        };

        let shard = match shard {
            Some(shard) => shard,
            None => return false,
        };

        // With this, killing will complete even if the promise returned is cancelled
        // (it should regardless, but just to play it safe..)
        let _ = tokio::spawn(async move {
            let _ = shard.kill().await;
        })
        .await;

        true
    }

    /// Kill everything and tidy up
    pub async fn shutdown(self) {
        // Spawn so we don't need to await cleanup if we don't care.
        // Run all kill futs simultaneously.
        let handle = tokio::spawn(async move {
            let (core, shards) = match self {
                Server::SingleProcessMode { core, .. }
                    => (core, DenseMap::new()),
                Server::ShardAndCoreMode { core, shards, ..}
                    => (core, shards),
                Server::ConnectToExistingMode { core, shards, .. }
                    => (core, shards)
            };

            let shard_kill_futs = shards.into_iter().map(|(_, s)| s.kill());
            let _ = tokio::join!(futures::future::join_all(shard_kill_futs), core.kill());
        });

        // You can wait for cleanup but aren't obliged to:
        let _ = handle.await;
    }

    /// Connect a new shard and return a process that you can interact with:
    pub async fn add_shard(&mut self) -> Result<ProcessId, Error> {
        match self {
            // Always get back the same "virtual" shard; we're always just talking to the core anyway.
            Server::SingleProcessMode { virtual_shard, .. } => {
                Ok(virtual_shard.id)
            },
            // We're connecting to an existing process. Find the next host we've been told about
            // round-robin style and use that as our new virtual shard.
            Server::ConnectToExistingMode { submit_hosts, next_submit_host_idx, shards, .. } => {
                let host = match submit_hosts.get(*next_submit_host_idx % submit_hosts.len()) {
                    Some(host) => host,
                    None => return Err(Error::CannotAddShard)
                };
                *next_submit_host_idx += 1;

                let pid = shards.add_with(|id| Process {
                    id,
                    host: format!("{}", host),
                    handle: None,
                    _channel_type: PhantomData,
                });

                Ok(pid)
            },
            // Start a new process and return that.
            Server::ShardAndCoreMode { shard_command, shards, core } => {
                // Where is the URI we'll want to submit things to?
                let core_shard_submit_uri = format!("http://{}/shard_submit", core.host);

                let mut shard_cmd: TokioCommand = shard_command.clone().into();
                shard_cmd
                    .arg("--listen")
                    .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
                    .arg("--log")
                    .arg("info")
                    .arg("--core")
                    .arg(core_shard_submit_uri)
                    .kill_on_drop(true)
                    .stdout(std::process::Stdio::piped())
                    .stdin(std::process::Stdio::piped());

                let mut shard_process = shard_cmd.spawn()?;
                let mut child_stdout = shard_process.stdout.take().expect("shard stdout");
                let shard_port = utils::get_port(&mut child_stdout)
                    .await
                    .map_err(|e| Error::ErrorObtainingPort(e))?;

                // Attempt to wait until we've received word that the shard is connected to the
                // core before continuing. If we don't wait for this, the connection may happen
                // after we've attempted to connect node sockets, and they would be booted and
                // made to reconnect, which we don't want to deal with in general.
                let _ = utils::wait_for_line_containing(
                    &mut child_stdout,
                    |s| s.contains("Connected to telemetry core"),
                    std::time::Duration::from_secs(5),
                )
                .await;

                // Since we're piping stdout from the child process, we need somewhere for it to go
                // else the process will get stuck when it tries to produce output:
                utils::drain(child_stdout, tokio::io::stderr());

                let pid = shards.add_with(|id| Process {
                    id,
                    host: format!("127.0.0.1:{}", shard_port),
                    handle: Some(shard_process),
                    _channel_type: PhantomData,
                });

                Ok(pid)
            },
        }
    }

    /// Start a server.
    pub async fn start(opts: StartOpts) -> Result<Server, Error> {
        let server = match opts {
            StartOpts::SingleProcess { command } => {
                let core_process = Server::start_core(command).await?;
                let virtual_shard_host = core_process.host.clone();
                Server::SingleProcessMode {
                    core: core_process,
                    virtual_shard: Process {
                        id: ProcessId(0),
                        host: virtual_shard_host,
                        handle: None,
                        _channel_type: PhantomData
                    }
                }
            },
            StartOpts::ShardAndCore { core_command, shard_command } => {
                let core_process = Server::start_core(core_command).await?;
                Server::ShardAndCoreMode {
                    core: core_process,
                    shard_command,
                    shards: DenseMap::new()
                }
            },
            StartOpts::ConnectToExisting { feed_host, submit_hosts } => {
                Server::ConnectToExistingMode {
                    submit_hosts,
                    next_submit_host_idx: 0,
                    shards: DenseMap::new(),
                    core: Process {
                        id: ProcessId(0),
                        host: feed_host,
                        handle: None,
                        _channel_type: PhantomData,
                    },
                }
            }
        };

        Ok(server)
    }

    /// Start up a core process and return it.
    async fn start_core(command: Command) -> Result<CoreProcess, Error> {
        let mut tokio_core_cmd: TokioCommand = command.into();
        let mut child = tokio_core_cmd
            .arg("--listen")
            .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
            .arg("--log")
            .arg("info")
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        // Find out the port that this is running on
        let mut child_stdout = child.stdout.take().expect("core stdout");
        let core_port = utils::get_port(&mut child_stdout)
            .await
            .map_err(|e| Error::ErrorObtainingPort(e))?;

        // Since we're piping stdout from the child process, we need somewhere for it to go
        // else the process will get stuck when it tries to produce output:
        utils::drain(child_stdout, tokio::io::stderr());

        let core_process = Process {
            id: ProcessId(0),
            host: format!("127.0.0.1:{}", core_port),
            handle: Some(child),
            _channel_type: PhantomData,
        };

        Ok(core_process)
    }
}

/// This represents a running process that we can connect to, which
/// may be either a `telemetry_shard` or `telemetry_core`.
pub struct Process<Channel> {
    id: ProcessId,
    /// Host that the process is running on (eg 127.0.0.1:8080).
    host: String,
    /// If we started the processes ourselves, we'll have a handle to
    /// them which we can use to kill them. Else, we may not.
    handle: Option<process::Child>,
    /// The kind of the process (lets us add methods specific to shard/core).
    _channel_type: PhantomData<Channel>,
}

/// A shard process with shard-specific methods.
pub type ShardProcess = Process<(channels::ShardSender, channels::ShardReceiver)>;

/// A core process with core-specific methods.
pub type CoreProcess = Process<(channels::FeedSender, channels::FeedReceiver)>;

impl<Channel> Process<Channel> {
    /// Get the ID of this process
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// Kill the process and wait for this to complete
    /// Not public: Klling done via Server.
    async fn kill(self) -> Result<(), Error> {
        match self.handle {
            Some(mut handle) => Ok(handle.kill().await?),
            None => Err(Error::CannotKillNoHandle),
        }
    }
}

impl<Send: From<ws_client::Sender>, Recv: From<ws_client::Receiver>> Process<(Send, Recv)> {
    /// Establish a connection to the process
    async fn connect_to_uri(&self, uri: &http::Uri) -> Result<(Send, Recv), Error> {
        ws_client::connect(uri)
            .await
            .map(|(s, r)| (s.into(), r.into()))
            .map_err(|e| e.into())
    }

    /// Establish multiple connections to the process
    async fn connect_multiple_to_uri(
        &self,
        uri: &http::Uri,
        num_connections: usize,
    ) -> Result<Vec<(Send, Recv)>, Error> {
        utils::connect_multiple_to_uri(uri, num_connections)
            .await
            .map(|v| v.into_iter().map(|(s, r)| (s.into(), r.into())).collect())
            .map_err(|e| e.into())
    }
}

impl ShardProcess {
    /// Establish a connection to the process
    pub async fn connect_node(&self) -> Result<(channels::ShardSender, channels::ShardReceiver), Error> {
        let uri = format!("http://{}/submit", self.host).parse()?;
        self.connect_to_uri(&uri).await
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple_nodes(&self, num_connections: usize) -> Result<Vec<(channels::ShardSender, channels::ShardReceiver)>, Error> {
        let uri = format!("http://{}/submit", self.host).parse()?;
        self.connect_multiple_to_uri(&uri, num_connections).await
    }
}

impl CoreProcess {
    /// Establish a connection to the process
    pub async fn connect_feed(&self) -> Result<(channels::FeedSender, channels::FeedReceiver), Error> {
        let uri = format!("http://{}/feed", self.host).parse()?;
        self.connect_to_uri(&uri).await
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple_feeds(&self, num_connections: usize) -> Result<Vec<(channels::FeedSender, channels::FeedReceiver)>, Error> {
        let uri = format!("http://{}/feed", self.host).parse()?;
        self.connect_multiple_to_uri(&uri, num_connections).await
    }
}

/// This defines a command to run. This exists because [`tokio::process::Command`]
/// cannot be cloned, but we need to be able to clone our command to spawn multiple
/// processes with it.
#[derive(Clone, Debug)]
pub struct Command {
    command: OsString,
    args: Vec<OsString>,
}

impl Command {
    pub fn new<S: Into<OsString>>(command: S) -> Command {
        Command {
            command: command.into(),
            args: Vec::new(),
        }
    }

    pub fn arg<S: Into<OsString>>(mut self, arg: S) -> Command {
        self.args.push(arg.into());
        self
    }
}

impl Into<TokioCommand> for Command {
    fn into(self) -> TokioCommand {
        let mut cmd = TokioCommand::new(self.command);
        cmd.args(self.args);
        cmd
    }
}
