import * as EventEmitter from 'events';
import Node from './node';
import { NodeId } from './nodeId';

export default class Aggregator extends EventEmitter {
    private nodes: Map<NodeId, Node> = new Map;

    public height: number = 0;

    add(node: Node) {
        this.nodes.set(node.id, node);
        node.once('disconnect', () => {
            node.removeAllListeners('block');

            this.nodes.delete(node.id);
        });

        node.on('block', () => this.updateBlock(node));
    }

    get nodeList(): Array<Node> {
        return Array.from(this.nodes.values());
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;

            console.log(`New block ${this.height}`);
        }

        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
