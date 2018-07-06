import * as EventEmitter from 'events';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import { timestamp, Types, FeedMessage } from '@dotstats/common';

export default class Chain {
    private nodes = new Set<Node>();
    private feeds = new FeedSet();

    public readonly events = new EventEmitter();
    public readonly label: Types.ChainLabel;

    public height = 0 as Types.BlockNumber;
    public blockTimestamp = 0 as Types.Timestamp;

    constructor(label: Types.ChainLabel) {
        this.label = label;
    }

    public get nodeCount(): number {
        return this.nodes.size;
    }

    public addNode(node: Node) {
        console.log(`[${this.label}] new node: ${node.name}`);

        this.nodes.add(node);
        this.feeds.broadcast(Feed.addedNode(node));

        node.events.once('disconnect', () => {
            node.events.removeAllListeners();

            this.nodes.delete(node);
            this.feeds.broadcast(Feed.removedNode(node));

            this.events.emit('disconnect', this.nodeCount);
        });

        node.events.on('block', () => this.updateBlock(node));
        node.events.on('stats', () => this.feeds.broadcast(Feed.stats(node)));
    }

    public addFeed(feed: Feed) {
        this.feeds.add(feed);

        // TODO: this is a bit unclean, find a better way
        feed.chain = this.label;

        feed.sendMessage(Feed.timeSync());
        feed.sendMessage(Feed.bestBlock(this.height, this.blockTimestamp));

        for (const node of this.nodes.values()) {
            feed.sendMessage(Feed.addedNode(node));
        }
    }

    public removeFeed(feed: Feed) {
        this.feeds.remove(feed);
    }

    public nodeList(): IterableIterator<Node> {
        return this.nodes.values();
    }

    public timeoutCheck() {
        const now = timestamp();

        for (const node of this.nodes.values()) {
            node.timeoutCheck(now);
        }

        this.feeds.broadcast(Feed.timeSync());
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;
            this.blockTimestamp = node.blockTimestamp;

            this.feeds.broadcast(Feed.bestBlock(this.height, this.blockTimestamp));

            console.log(`[${this.label}] New block ${this.height}`);
        }

        this.feeds.broadcast(Feed.imported(node));

        console.log(`[${this.label}] ${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
