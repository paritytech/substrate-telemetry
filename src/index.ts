import * as WebSocket from 'ws';
import Node from './node';

const wss = new WebSocket.Server({ port: 1024 });
const nodes = new WeakSet();

wss.on('connection', async (socket: WebSocket) => {
    let node: Node;

    try {
        node = await Node.fromSocket(socket);
    } catch (err) {
        console.error(err);

        return;
    }

    nodes.add(node);
    node.once('disconnect', () => nodes.delete(node));
});
