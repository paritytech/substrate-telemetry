# Roadmap

## Frontend

- Provide a pie chart of node implementations/versions above the list, next to Last Block timer, to have an overview of what the dominant impl/version is, and whether there is a significant amount of nodes running older versions.

    Latest partial implementation of this can be found in the `mh-piechart-new` branch.

- Provide a block propagation graph: X axis being a number of blocks a node is behind, while Y axis is the number of nodes in that bucket. The size of buckets should grow: [0, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000]

    Partial implementation of this can be found in `mh-version-piecharts` branch, although that branch is pre FE refactoring, so starting a new one and splitting it from pie charts would be advised.

- Network search by name when viewing the complete list (#276).

## Backend

- To keep up with increasing traffic, we should split out a new service from the current backend that replaces the `/submit` endpoint. This new service should take ownership of JSON deserialization of incoming messages from the nodes, discarding messages that Telemetry does not need, resolving chain multiplexing (this will likely need some two-way communication with the main backend when a new node connects), and then forwarding those messages using a lightweight protocol (Cap'n Proto or Protocol Buffers) to the main telemetry backend. Unlike the backend, which needs to have a single instance to keep track of all state changes, this new service should be stateless and therefore we should be able to spawn multiple instances of it behind a load balancer. This would solve the two bottlenecks we're currently having: the number of concurrent connections going to the backend, and the CPU use that comes from IO switching and JSON deserialization.
