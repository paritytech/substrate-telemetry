import * as WebSocket from 'ws';
import Node from './node';

const wss = new WebSocket.Server({ port: 1024 });

wss.on('connection', async (socket: WebSocket) => {
    await Node.fromSocket(socket);
});
