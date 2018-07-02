import * as WebSocket from 'ws';
import * as express from 'express';
import { createServer } from 'http';
import Node from './node';
import Feed from './feed';
import Aggregator from './aggregator';

const aggregator = new Aggregator;
const app = express();
const server = createServer(app);

// WebSocket for Nodes feeding telemetry data to the server
const incomingTelemetry = new WebSocket.Server({ port: 1024 });

// WebSocket for web clients listening to the telemetry data aggregate
const telemetryFeed = new WebSocket.Server({ server });

app.get('/', function (req, res) {
    res.send('See live listing at <a href="http://telemetry.polkadot.io/">https://telemetry.polkadot.io/<a>');
});

incomingTelemetry.on('connection', async (socket: WebSocket) => {
    try {
        aggregator.addNode(await Node.fromSocket(socket));
    } catch (err) {
        console.error(err);
    }
});

telemetryFeed.on('connection', (socket: WebSocket) => {
    aggregator.addFeed(new Feed(socket));
});

console.log('Starting server on port 8080');
server.listen(8080);
