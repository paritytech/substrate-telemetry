import * as WebSocket from 'ws';
import * as express from 'express';
import { createServer } from 'http';
import Node from './node';
import Aggregator from './aggregator';

const aggregator = new Aggregator;
const app = express();
const server = createServer(app);

// WebSocket for Nodes feeding telemetry data to the server
const incomingTelemetry = new WebSocket.Server({ port: 1024 });

// WebSocket for web clients listening to the telemetry data aggregate
const telemetryFeed = new WebSocket.Server({ server });

app.get('/', function (req, res) {
    const nodes = Array
                    .from(aggregator.nodes)
                    .map((node: Node) => `${node.name} | ${node.height} | Block time ${node.blockTime / 1000}s`);

    res.send(

`<pre>
Best block: ${aggregator.height}

Node list:
${nodes.join('\n')}
</pre>`

    );
});

incomingTelemetry.on('connection', async (socket: WebSocket) => {
    try {
        aggregator.add(await Node.fromSocket(socket));
    } catch (err) {
        console.error(err);

        return;
    }
});

telemetryFeed.on('connection', (socket: WebSocket) => {
    socket.send('HELLO THAR!');
    socket.close();
});

server.listen(8080);
