use crate::ws_client::{ Sender, Receiver };

/// We either say where to conenct to, or we start the binaries
/// ourselves. Either way, we hand back a `Connection` object
/// which allows us to talk to the running instances.
pub enum Opts {
    StartProcesses {
        shard_command: Option<String>,
        num_shards: usize,
        telemetry_command: Option<String>
    },
    ConnectToExisting {
        shard_uris: Vec<http::Uri>,
        telemetry_uri: http::Uri
    }
}

pub struct Connection {
    shard_sockets: Vec<(Sender, Receiver)>,
    telemetry_socket: Vec<(Sender, Receiver)>
}