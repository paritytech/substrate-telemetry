use tokio::process::Command;
use crate::connect_to_servers::{ Server, StartProcessOpts, Process };

pub struct Runner {
    shard_command: Option<Command>,
    core_command: Option<Command>
}

impl Runner {
    pub fn new() -> Runner {
        Runner {
            shard_command: None,
            core_command: None
        }
    }

    pub fn shard_command(mut self, cmd: Command) -> Self {
        self.shard_command = Some(cmd);
        self
    }

    pub fn core_command(mut self, cmd: Command) -> Self {
        self.core_command = Some(cmd);
        self
    }

    pub async fn build(self) -> Result<Processes, anyhow::Error> {
        let mut server = Server::start_processes(StartProcessOpts {
            shard_command: self.shard_command,
            num_shards: 1,
            core_command: self.core_command,
        }).await?;

        let core_process = server.core;
        let shard_process = server.shards.remove(0);

        Ok(Processes {
            core_process,
            shard_process,
        })
    }
}

/// A representation of the running processes that we can connect and send messages to.
pub struct Processes {
    shard_process: Process,
    core_process: Process,
}

impl Processes {
    pub async fn cleanup(self) {
        let handle = tokio::spawn(async move {
            let _ = tokio::join!(
                self.shard_process.kill(),
                self.core_process.kill()
            );
        });

        // You can wait for cleanup but aren't obliged to:
        let _ = handle.await;
    }
}
