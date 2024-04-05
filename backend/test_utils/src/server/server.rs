// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
        /// Log output from started processes to stderr?
        log_output: bool,
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
        /// Log output from started processes to stderr?
        log_output: bool,
    },
    /// Connect to existing process(es).
    ConnectToExisting {
        /// Where are the processes that we can `/submit` things to?
        /// Eg: `vec![127.0.0.1:12345, 127.0.0.1:9091]`
        submit_hosts: Vec<String>,
        /// Where is the process that we can subscribe to the `/feed` of?
        /// Eg: `127.0.0.1:3000`
        feed_host: String,
        /// Log output from started processes to stderr?
        log_output: bool,
    },
}

/// This represents a telemetry server. It can be in different modes
/// depending on how it was started, but the interface is similar in every case
/// so that tests are somewhat compatible with multiple configurations.
pub struct Server {
    /// Should we log output from the processes we start?
    log_output: bool,
    /// Core process that we can connect to.
    core: CoreProcess,
    /// Things that vary based on the mode we are in.
    mode: ServerMode,
}
pub enum ServerMode {
    SingleProcessMode {
        /// A virtual shard that we can hand out.
        virtual_shard: ShardProcess,
    },
    ShardAndCoreMode {
        /// Command to run to start a new shard.
        shard_command: Command,
        /// Shard processes that we can connect to.
        shards: DenseMap<ProcessId, ShardProcess>,
    },
    ConnectToExistingMode {
        /// The hosts that we can connect to submit things.
        submit_hosts: Vec<String>,
        /// Which host do we use next (we'll cycle around them
        /// as shards are "added").
        next_submit_host_idx: usize,
        /// Shard processes that we can connect to.
        shards: DenseMap<ProcessId, ShardProcess>,
    },
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
    InvalidUri(#[from] http::uri::InvalidUri),
}

impl Server {
    pub fn get_core(&self) -> &CoreProcess {
        &self.core
    }

    pub fn get_shard(&self, id: ProcessId) -> Option<&ShardProcess> {
        match &self.mode {
            ServerMode::SingleProcessMode { virtual_shard, .. } => Some(virtual_shard),
            ServerMode::ShardAndCoreMode { shards, .. } => shards.get(id),
            ServerMode::ConnectToExistingMode { shards, .. } => shards.get(id),
        }
    }

    pub async fn kill_shard(&mut self, id: ProcessId) -> bool {
        let shard = match &mut self.mode {
            // Can't remove the pretend shard:
            ServerMode::SingleProcessMode { .. } => return false,
            ServerMode::ShardAndCoreMode { shards, .. } => shards.remove(id),
            ServerMode::ConnectToExistingMode { shards, .. } => shards.remove(id),
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
            let core = self.core;
            let shards = match self.mode {
                ServerMode::SingleProcessMode { .. } => DenseMap::new(),
                ServerMode::ShardAndCoreMode { shards, .. } => shards,
                ServerMode::ConnectToExistingMode { shards, .. } => shards,
            };

            let shard_kill_futs = shards.into_iter().map(|(_, s)| s.kill());
            let _ = tokio::join!(futures::future::join_all(shard_kill_futs), core.kill());
        });

        // You can wait for cleanup but aren't obliged to:
        let _ = handle.await;
    }

    /// Connect a new shard and return a process that you can interact with:
    pub async fn add_shard(&mut self) -> Result<ProcessId, Error> {
        match &mut self.mode {
            // Always get back the same "virtual" shard; we're always just talking to the core anyway.
            ServerMode::SingleProcessMode { virtual_shard, .. } => Ok(virtual_shard.id),
            // We're connecting to an existing process. Find the next host we've been told about
            // round-robin style and use that as our new virtual shard.
            ServerMode::ConnectToExistingMode {
                submit_hosts,
                next_submit_host_idx,
                shards,
                ..
            } => {
                let host = match submit_hosts.get(*next_submit_host_idx % submit_hosts.len()) {
                    Some(host) => host,
                    None => return Err(Error::CannotAddShard),
                };
                *next_submit_host_idx += 1;

                let pid = shards.add_with(|id| Process {
                    id,
                    host: format!("{}", host),
                    handle: None,
                    _channel_type: PhantomData,
                });

                Ok(pid)
            }
            // Start a new process and return that.
            ServerMode::ShardAndCoreMode {
                shard_command,
                shards,
            } => {
                // Where is the URI we'll want to submit things to?
                let core_shard_submit_uri = format!("http://{}/shard_submit", self.core.host);

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
                if self.log_output {
                    utils::drain(child_stdout, tokio::io::stderr());
                } else {
                    utils::drain(child_stdout, tokio::io::sink());
                }

                let pid = shards.add_with(|id| Process {
                    id,
                    host: format!("127.0.0.1:{}", shard_port),
                    handle: Some(shard_process),
                    _channel_type: PhantomData,
                });

                Ok(pid)
            }
        }
    }

    /// Start a server.
    pub async fn start(opts: StartOpts) -> Result<Server, Error> {
        let server = match opts {
            StartOpts::SingleProcess {
                command,
                log_output,
            } => {
                let core_process = Server::start_core(log_output, command).await?;
                let virtual_shard_host = core_process.host.clone();
                Server {
                    log_output,
                    core: core_process,
                    mode: ServerMode::SingleProcessMode {
                        virtual_shard: Process {
                            id: ProcessId(0),
                            host: virtual_shard_host,
                            handle: None,
                            _channel_type: PhantomData,
                        },
                    },
                }
            }
            StartOpts::ShardAndCore {
                core_command,
                shard_command,
                log_output,
            } => {
                let core_process = Server::start_core(log_output, core_command).await?;
                Server {
                    log_output,
                    core: core_process,
                    mode: ServerMode::ShardAndCoreMode {
                        shard_command,
                        shards: DenseMap::new(),
                    },
                }
            }
            StartOpts::ConnectToExisting {
                feed_host,
                submit_hosts,
                log_output,
            } => Server {
                log_output,
                core: Process {
                    id: ProcessId(0),
                    host: feed_host,
                    handle: None,
                    _channel_type: PhantomData,
                },
                mode: ServerMode::ConnectToExistingMode {
                    submit_hosts,
                    next_submit_host_idx: 0,
                    shards: DenseMap::new(),
                },
            },
        };

        Ok(server)
    }

    /// Start up a core process and return it.
    async fn start_core(log_output: bool, command: Command) -> Result<CoreProcess, Error> {
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
        if log_output {
            utils::drain(child_stdout, tokio::io::stderr());
        } else {
            utils::drain(child_stdout, tokio::io::sink());
        }

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

    /// Get the host that this process is running on
    pub fn host(&self) -> &str {
        &self.host
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

/// Establish a raw WebSocket connection (not cancel-safe)
async fn connect_to_uri_raw(
    uri: &http::Uri,
) -> Result<(ws_client::RawSender, ws_client::RawReceiver), Error> {
    ws_client::connect(uri)
        .await
        .map(|c| c.into_raw())
        .map_err(|e| e.into())
}

impl<Send: From<ws_client::Sender>, Recv: From<ws_client::Receiver>> Process<(Send, Recv)> {
    /// Establish a connection to the process
    async fn connect_to_uri(uri: &http::Uri) -> Result<(Send, Recv), Error> {
        ws_client::connect(uri)
            .await
            .map(|c| c.into_channels())
            .map(|(s, r)| (s.into(), r.into()))
            .map_err(|e| e.into())
    }

    /// Establish multiple connections to the process
    async fn connect_multiple_to_uri(
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
    /// Establish a raw connection to the process
    pub async fn connect_node_raw(
        &self,
    ) -> Result<(ws_client::RawSender, ws_client::RawReceiver), Error> {
        let uri = format!("http://{}/submit", self.host).parse()?;
        connect_to_uri_raw(&uri).await
    }

    /// Establish a connection to the process
    pub async fn connect_node(
        &self,
    ) -> Result<(channels::ShardSender, channels::ShardReceiver), Error> {
        let uri = format!("http://{}/submit", self.host).parse()?;
        Process::connect_to_uri(&uri).await
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple_nodes(
        &self,
        num_connections: usize,
    ) -> Result<Vec<(channels::ShardSender, channels::ShardReceiver)>, Error> {
        let uri = format!("http://{}/submit", self.host).parse()?;
        Process::connect_multiple_to_uri(&uri, num_connections).await
    }
}

impl CoreProcess {
    /// Establish a raw connection to the process
    pub async fn connect_feed_raw(
        &self,
    ) -> Result<(ws_client::RawSender, ws_client::RawReceiver), Error> {
        let uri = format!("http://{}/feed", self.host).parse()?;
        connect_to_uri_raw(&uri).await
    }

    /// Establish a connection to the process
    pub async fn connect_feed(
        &self,
    ) -> Result<(channels::FeedSender, channels::FeedReceiver), Error> {
        let uri = format!("http://{}/feed", self.host).parse()?;
        Process::connect_to_uri(&uri).await
    }

    /// Establish multiple connections to the process
    pub async fn connect_multiple_feeds(
        &self,
        num_connections: usize,
    ) -> Result<Vec<(channels::FeedSender, channels::FeedReceiver)>, Error> {
        let uri = format!("http://{}/feed", self.host).parse()?;
        Process::connect_multiple_to_uri(&uri, num_connections).await
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
