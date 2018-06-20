import * as WebSocket from 'ws';
import Node from './node';
import Aggregator from './aggregator';

const wss = new WebSocket.Server({ port: 1024 });

const aggregator = new Aggregator;

wss.on('connection', async (socket: WebSocket) => {
    try {
        aggregator.add(await Node.fromSocket(socket));
    } catch (err) {
        console.error(err);

        return;
    }
});
