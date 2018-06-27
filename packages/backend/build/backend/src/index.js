"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const WebSocket = require("ws");
const express = require("express");
const http_1 = require("http");
const node_1 = require("./node");
const feed_1 = require("./feed");
const aggregator_1 = require("./aggregator");
const shared_1 = require("@dotstats/shared");
const aggregator = new aggregator_1.default;
const app = express();
const server = http_1.createServer(app);
// WebSocket for Nodes feeding telemetry data to the server
const incomingTelemetry = new WebSocket.Server({ port: 1024 });
// WebSocket for web clients listening to the telemetry data aggregate
const telemetryFeed = new WebSocket.Server({ server });
app.get('/', function (req, res) {
    function nodeInfo(node) {
        return `${node.name} | ${node.height} | Block time ${node.blockTime / 1000}s`;
    }
    res.send(`<pre>
Best block: ${aggregator.height}

Node list:
${shared_1.join(shared_1.map(aggregator.nodeList(), nodeInfo), '\n')}
</pre>`);
});
incomingTelemetry.on('connection', async (socket) => {
    try {
        aggregator.addNode(await node_1.default.fromSocket(socket));
    }
    catch (err) {
        console.error(err);
    }
});
telemetryFeed.on('connection', (socket) => {
    aggregator.addFeed(new feed_1.default(socket));
});
console.log('Starting server on port 8080');
server.listen(8080);
//# sourceMappingURL=index.js.map