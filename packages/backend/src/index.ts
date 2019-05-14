import * as http from 'http';
import * as url from 'url';
import * as WebSocket from 'ws';
import Node from './Node';
import Feed from './Feed';
import Aggregator from './Aggregator';
import {Types} from '@dotstats/common';

const WS_PORT_TELEMETRY_SERVER = Number(process.env.TELEMETRY_SERVER || 1024);
const WS_PORT_FEED_SERVER = Number(process.env.FEED_SERVER || 8080);

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

  console.log(`[System] ${node} open telemetry connections; ${feed} open feed connections`);

  setTimeout(logClients, 5000);
}

logClients();

telemetryFeed.on('connection', (socket: WebSocket) => {
  aggregator.addFeed(new Feed(socket));
});

http.createServer((request, response) => {
  const incoming_url = request.url || "";
  const parsed_url = url.parse(incoming_url, true);
  const path = decodeURI(parsed_url.path || "");
  if (path.startsWith("/network_state/")) {
    const [chainLabel, strNodeId] = path.split('/').slice(2);
    const chain = aggregator.getExistingChain(chainLabel as Types.ChainLabel);
    if (chain) {
      const nodeList = Array.from(chain.nodeList());
      const nodeId = Number(strNodeId);
      const node = nodeList.filter((node) => node.id == nodeId)[0];
      if (node && node.networkState) {
        const { networkState } = node;

        response.writeHead(200, {"Content-Type": "application/json"});
        response.write(typeof networkState === 'string' ? networkState : JSON.stringify(networkState));
      } else {
        response.writeHead(404, {"Content-Type": "text/plain"});
        response.write("Node has disconnected or has not submitted its network state yet");
      }
    }
  }
  response.end();
}).listen(8081);
