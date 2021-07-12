use std::ffi::OsString;
use std::marker::PhantomData;
use crate::ws_client;
use tokio::process::{ self, Command as TokioCommand };
use super::{ channels, utils };
use common::{ id_type, DenseMap };

id_type! {
    /// The ID of a running process. Cannot be constructed externally.
    pub struct ProcessId(usize);
}

pub struct StartOpts {
    /// Optional command to run to start a shard (instead of `telemetry_shard`).
    /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
    pub shard_command: Option<Command>,
    /// Optional command to run to start a telemetry core process (instead of `telemetry_core`).
    /// The `--listen` and `--log` arguments will be appended within and shouldn't be provided.
    pub core_command: Option<Command>
}

impl Default for StartOpts {
    fn default() -> Self {
        StartOpts {
            shard_command: None,
            core_command: None
        }
    }
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
    CannotKillNoHandle,
    #[error("Whoops; attempt to add a shard to a server we didn't start (and so have no handle to)")]
    CannotAddShardNoHandle,
}

/// This provides back connections (or groups of connections) that are
/// hooked up to the running processes and ready to send/receive messages.
pub struct Server {
    /// URI to connect a shard to core:
    core_shard_submit_uri: Option<http::Uri>,
    /// Command to run to start a new shard:
    shard_command: Option<Command>,
    /// Shard processes that we can connect to
    shards: DenseMap<ProcessId, ShardProcess>,
    /// Core process that we can connect to
    core: CoreProcess,
}

impl Server {
    pub fn get_core(&self) -> &CoreProcess {
        &self.core
    }

    pub fn get_shard(&self, id: ProcessId) -> Option<&ShardProcess> {
        self.shards.get(id)
    }

    pub fn iter_shards(&self) -> impl Iterator<Item = &ShardProcess> {
        self.shards.iter().map(|(_,v)| v)
    }

    pub async fn kill_shard(&mut self, id: ProcessId) -> bool {
        let shard = match self.shards.remove(id) {
            Some(shard) => shard,
            None => return false
        };

        // With this, killing will complete even if the promise returned is cancelled
        // (it should regardless, but just to play it safe..)
        let _ = tokio::spawn(async move {
            let _ = shard.kill().await;
        }).await;

        true
    }

    /// Kill everything and tidy up
    pub async fn shutdown(self) {
        // Spawn so we don't need to await cleanup if we don't care.
        // Run all kill futs simultaneously.
        let handle = tokio::spawn(async move {
            let shard_kill_futs = self.shards
                .into_iter()
                .map(|(_,s)| s.kill());

            let _ = tokio::join!(
                futures::future::join_all(shard_kill_futs),
                self.core.kill()
            );
        });

        // You can wait for cleanup but aren't obliged to:
        let _ = handle.await;
    }

    /// Connect a new shard and return a process that you can interact with:
    pub async fn add_shard(&mut self) -> Result<ProcessId, Error> {
        let core_uri = match &self.core_shard_submit_uri {
            Some(uri) => uri,
            None => return Err(Error::CannotAddShardNoHandle)
        };

        let mut shard_cmd: TokioCommand = match &self.shard_command {
            Some(cmd) => cmd.clone(),
            None => super::default_commands::default_telemetry_shard_command()?
        }.into();

        shard_cmd
            .arg("--listen")
            .arg("127.0.0.1:0") // 0 to have a port picked by the kernel
            .arg("--log")
            .arg("info")
            .arg("--core")
            .arg(core_uri.to_string())
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
            "Connected to telemetry core",
            std::time::Duration::from_secs(5)
        ).await;

        // Since we're piping stdout from the child process, we need somewhere for it to go
        // else the process will get stuck when it tries to produce output:
        utils::drain(child_stdout, tokio::io::sink());

        let shard_uri = format!("http://127.0.0.1:{}/submit", shard_port)
            .parse()
            .expect("valid submit URI");

        let pid = self.shards.add_with(|id| Process {
            id,
            handle: Some(shard_process),
            uri: shard_uri,
            _channel_type: PhantomData
        });

        Ok(pid)
    }

    /// Start a telemetry_core process with default opts. From here, we can add/remove shards as needed.
    pub async fn start_default() -> Result<Server, Error> {
        Server::start(StartOpts::default()).await
    }

    /// Start a telemetry_core process. From here, we can add/remove shards as needed.
    pub async fn start(opts: StartOpts) -> Result<Server, Error> {

        let mut core_cmd: TokioCommand = match opts.core_command {
            Some(cmd) => cmd,
            None => super::default_commands::default_telemetry_core_command()?
        }.into();

        let mut child = core_cmd
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
        utils::drain(child_stdout, tokio::io::sink());

        // URI for feeds to connect to the core:
        let feed_uri = format!("http://127.0.0.1:{}/feed", core_port)
            .parse()
            .expect("valid feed URI");

        Ok(Server {
            shard_command: opts.shard_command,
            core_shard_submit_uri: Some(format!("http://127.0.0.1:{}/shard_submit", core_port)
                .parse()
                .expect("valid shard_submit URI")),
            shards: DenseMap::new(),
            core: Process {
                id: ProcessId(0),
                handle: Some(child),
                uri: feed_uri,
                _channel_type: PhantomData,
            }
        })
    }

    /// Establshes the requested connections to existing processes.
    pub fn connect_to_existing(opts: ConnectToExistingOpts) -> Server {
        let mut shards = DenseMap::new();
        for shard_uri in opts.shard_uris {
            shards.add_with(|id| Process {
                id,
                uri: shard_uri,
                handle: None,
                _channel_type: PhantomData,
            });
        }

        Server {
            shard_command: None,
            // We can't add shards if starting in this mode:
            core_shard_submit_uri: None,
            shards,
            core: Process {
                id: ProcessId(0),
                uri: opts.feed_uri,
                handle: None,
                _channel_type: PhantomData,
            }
        }
    }
}


/// This represents a running process that we can connect to, which
/// may be either a `telemetry_shard` or `telemetry_core`.
pub struct Process<Channel> {
    id: ProcessId,
    /// If we started the processes ourselves, we'll have a handle to
    /// them which we can use to kill them. Else, we may not.
    handle: Option<process::Child>,
    /// The URI that we can use to connect to the process socket.
    uri: http::Uri,
    /// The kind of the process (lets us add methods specific to shard/core).
    _channel_type: PhantomData<Channel>
}

/// A shard process with shard-specific methods.
pub type ShardProcess = Process<(channels::ShardSender, channels::ShardReceiver)>;

/// A core process with core-specific methods.
pub type CoreProcess = Process<(channels::FeedSender, channels::FeedReceiver)>;

impl <Channel> Process<Channel> {
    /// Get the ID of this process
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// Kill the process and wait for this to complete
    /// Not public: Klling done via Server.
    async fn kill(self) -> Result<(), Error> {
        match self.handle {
            Some(mut handle) => Ok(handle.kill().await?),
            None => Err(Error::CannotKillNoHandle)
        }
    }
}

impl <Send: From<ws_client::Sender>, Recv: From<ws_client::Receiver>> Process<(Send, Recv)> {
    /// Establish a connection to the process
    pub async fn connect(&self) -> Result<(Send, Recv), Error> {
        ws_client::connect(&self.uri)
            .await
            .map(|(s,r)| (s.into(), r.into()))
            .map_err(|e| e.into())
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple(&self, num_connections: usize) -> Result<Vec<(Send, Recv)>, Error> {
        utils::connect_multiple_to_uri(&self.uri, num_connections)
            .await
            .map(|v| v.into_iter().map(|(s,r)| (s.into(), r.into())).collect())
            .map_err(|e| e.into())
    }
}

/// This defines a command to run. This exists because [`tokio::process::Command`]
/// cannot be cloned, but we need to be able to clone our command to spawn multiple
/// processes with it.
#[derive(Clone, Debug)]
pub struct Command {
    command: OsString,
    args: Vec<OsString>
}

impl Command {
    pub fn new<S: Into<OsString>>(command: S) -> Command {
        Command {
            command: command.into(),
            args: Vec::new()
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