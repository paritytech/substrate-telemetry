# Backend Crates

This folder contains the rust crates and documentation specific to the telemetry backend. A description of the folders:

- [telemetry](./telemetry): The Telemetry Core. This aggregates data received from shards and allows UI feeds to connect and receive this information.
- [shard](./shard): A Shard. It's expected that multiple of these will run. Nodes will connect to Shard instances and send JSON telemetry to them, and Shard instances will each connect to the Telemetry Core and relay on relevant data to it.
- [common](./common): common code shared between the telemetry shard and core
- [docs](./docs): Material supporting the documentation lives here

# Architecture

As we move to a sharded version of this telemetry server, this set of architecture diagrams may be useful in helping to understand the current setup (middle diagram), previous setup (first diagram) and possible future setup if we need to scale further (last diagram):

![Architecture Diagram](./docs/architecture.svg)

# Deployment

A `shard.Dockerfile` and `telemetry.Dockerfile` exist to build the Shard and Telemetry Core binaries into docker containers. A `docker-compose.yaml` in the root of the repository can serve as an example of these services, along with the UI, running together.