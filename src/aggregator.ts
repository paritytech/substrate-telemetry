import * as EventEmitter from 'events';
import Node from './node';

export default class Aggregator extends EventEmitter {
    private nodes: WeakSet<Node> = new WeakSet;
    private height: number = 0;

    add(node: Node) {
        this.nodes.add(node);
        node.once('disconnect', () => {
            node.removeAllListeners('block');

            this.nodes.delete(node);
        });

        node.on('block', () => this.updateBlock(node));
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;

            console.log(`New block ${this.height}`);
        }

        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
