import * as EventEmitter from 'events';
import Node from './node';
import { NodeId } from './nodeId';

export default class Aggregator extends EventEmitter {
    private _nodes: Map<NodeId, Node> = new Map;

    public height: number = 0;

    constructor() {
        super();

        setInterval(() => this.timeoutCheck(), 10000);
    }

    public add(node: Node) {
        this._nodes.set(node.id, node);
        node.once('disconnect', () => {
            node.removeAllListeners('block');

            this._nodes.delete(node.id);
        });

        node.on('block', () => this.updateBlock(node));
    }

    public get nodes(): IterableIterator<Node> {
        return this._nodes.values();
    }

    public get length(): number {
        return this._nodes.size;
    }

    private timeoutCheck() {
        const now = Date.now();

        for (const node of this.nodes) {
            node.timeoutCheck(now);
        }
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;

            console.log(`New block ${this.height}`);
        }

        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
