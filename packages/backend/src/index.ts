import * as WebSocket from 'ws';
import Node from './Node';
import Feed from './Feed';
import Aggregator from './Aggregator';

const WS_PORT_TELEMETRY_SERVER = 1024;
const WS_PORT_FEED_SERVER = 8080;

const aggregator = new Aggregator();

// WebSocket for Nodes feeding telemetry data to the server
const incomingTelemetry = new WebSocket.Server({ port: WS_PORT_TELEMETRY_SERVER });

// WebSocket for web clients listening to the telemetry data aggregate
const telemetryFeed = new WebSocket.Server({ port: WS_PORT_FEED_SERVER });

console.log(`Telemetry server listening on port ${WS_PORT_TELEMETRY_SERVER}`);
console.log(`Feed server listening on port ${WS_PORT_FEED_SERVER}`);

const ipv4 = /[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/;

incomingTelemetry.on('connection', async (socket, req) => {
  try {
    const [ ip ] = (req.headers['x-forwarded-for'] || req.connection.remoteAddress || '0.0.0.0')
      .toString()
      .match(ipv4) || ['0.0.0.0'];

    const node = await Node.fromSocket(socket, ip);

    aggregator.addNode(node);
  } catch (err) {
    console.error(err);
  }
});

function logClients() {
  const feed = telemetryFeed.clients.size;
  const node = incomingTelemetry.clients.size;

  console.log(`[System] ${feed} open telemetry connections; ${node} open feed connections`);

  setTimeout(logClients, 5000);
}

logClients();

telemetryFeed.on('connection', (socket: WebSocket) => {
  aggregator.addFeed(new Feed(socket));
});

