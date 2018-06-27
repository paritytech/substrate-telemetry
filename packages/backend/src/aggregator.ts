import * as EventEmitter from 'events';
import Node from './node';
import Feed, { FeedData } from './feed';
import { Id, IdSet } from '@dotstats/common';

export default class Aggregator extends EventEmitter {
    private nodes: IdSet<Node> = new IdSet<Node>();
    private feeds: IdSet<Feed> = new IdSet<Feed>();

    public height: number = 0;

    constructor() {
        super();

        setInterval(() => this.timeoutCheck(), 10000);
    }

    public addNode(node: Node) {
        this.nodes.add(node);
        this.broadcast(Feed.addedNode(node));

        node.once('disconnect', () => {
            node.removeAllListeners('block');

            this.nodes.remove(node);
            this.broadcast(Feed.removedNode(node));
        });

        node.on('block', () => this.updateBlock(node));
    }

    public addFeed(feed: Feed) {
        this.feeds.add(feed);

        feed.send(Feed.bestBlock(this.height));

        for (const node of this.nodes.entries) {
            feed.send(Feed.addedNode(node));
        }

        feed.once('disconnect', () => {
            this.feeds.remove(feed);
        })
    }

    public nodeList(): IterableIterator<Node> {
        return this.nodes.entries;
    }

    private broadcast(data: FeedData) {
        for (const feed of this.feeds.entries) {
            feed.send(data);
        }
    }

    private timeoutCheck() {
        const now = Date.now();

        for (const node of this.nodes.entries) {
            node.timeoutCheck(now);
        }
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;

            this.broadcast(Feed.bestBlock(this.height));

            console.log(`New block ${this.height}`);
        }

        this.broadcast(Feed.imported(node));

        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
