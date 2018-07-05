import * as EventEmitter from 'events';
import Node from './node';
import Feed from './feed';
import { timestamp, Types, IdSet, FeedMessage } from '@dotstats/common';

export default class Aggregator extends EventEmitter {
    private nodes = new IdSet<Types.NodeId, Node>();
    private feeds = new IdSet<Types.FeedId, Feed>();
    private messages: Array<FeedMessage.Message> = [];

    public height = 0 as Types.BlockNumber;
    public blockTimestamp = 0 as Types.Timestamp;

    constructor() {
        super();

        setInterval(() => this.timeoutCheck(), 10000);
    }

    public addNode(node: Node) {
        this.nodes.add(node);
        this.broadcast(Feed.addedNode(node));

        node.once('disconnect', () => {
            node.removeAllListeners();

            this.nodes.remove(node);
            this.broadcast(Feed.removedNode(node));
        });

        node.on('block', () => this.updateBlock(node));
        node.on('stats', () => this.broadcast(Feed.stats(node)));
    }

    public addFeed(feed: Feed) {
        this.feeds.add(feed);

        const messages = [Feed.timeSync(), Feed.bestBlock(this.height, this.blockTimestamp)];

        for (const node of this.nodes.values()) {
            messages.push(Feed.addedNode(node));
        }

        feed.sendMessages(messages);

        feed.once('disconnect', () => {
            this.feeds.remove(feed);
        });
    }

    public nodeList(): IterableIterator<Node> {
        return this.nodes.values();
    }

    private broadcast(message: FeedMessage.Message) {
        const queue = this.messages.length === 0;

        this.messages.push(message);

        if (queue) {
            process.nextTick(() => {
                const data = FeedMessage.serialize(this.messages);
                this.messages = [];

                for (const feed of this.feeds.values()) {
                    feed.sendData(data);
                }
            });
        }
    }

    private timeoutCheck() {
        const now = timestamp();

        for (const node of this.nodes.values()) {
            node.timeoutCheck(now);
        }

        this.broadcast(Feed.timeSync());
    }

    private updateBlock(node: Node) {
        if (node.height > this.height) {
            this.height = node.height;
            this.blockTimestamp = node.blockTimestamp;

            this.broadcast(Feed.bestBlock(this.height, this.blockTimestamp));

            console.log(`New block ${this.height}`);
        }

        this.broadcast(Feed.imported(node));

        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
