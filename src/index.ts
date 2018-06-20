import * as WebSocket from 'ws';
import * as express from 'express';
import Node from './node';
import Aggregator from './aggregator';

const wss = new WebSocket.Server({ port: 1024 });

const aggregator = new Aggregator;
const app = express();

// respond with "hello world" when a GET request is made to the homepage
app.get('/', function (req, res) {
    const nodes = aggregator
        .nodeList
        .map((node: Node) => `${node.name} | ${node.height} | Block time ${node.blockTime / 1000}s`);

    res.send(

`<pre>
Best block: ${aggregator.height}

Node list:
${nodes.join('\n')}
</pre>`

    );
});

app.listen(8080);

wss.on('connection', async (socket: WebSocket) => {
    try {
        aggregator.add(await Node.fromSocket(socket));
    } catch (err) {
        console.error(err);

        return;
    }
});
