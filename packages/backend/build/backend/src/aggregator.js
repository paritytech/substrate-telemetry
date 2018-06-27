"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const EventEmitter = require("events");
const feed_1 = require("./feed");
const shared_1 = require("@dotstats/shared");
class Aggregator extends EventEmitter {
    constructor() {
        super();
        this.nodes = new shared_1.IdSet();
        this.feeds = new shared_1.IdSet();
        this.height = 0;
        setInterval(() => this.timeoutCheck(), 10000);
    }
    addNode(node) {
        this.nodes.add(node);
        this.broadcast(feed_1.default.addedNode(node));
        node.once('disconnect', () => {
            node.removeAllListeners('block');
            this.nodes.remove(node);
            this.broadcast(feed_1.default.removedNode(node));
        });
        node.on('block', () => this.updateBlock(node));
    }
    addFeed(feed) {
        this.feeds.add(feed);
        feed.send(feed_1.default.bestBlock(this.height));
        for (const node of this.nodes.entries) {
            feed.send(feed_1.default.addedNode(node));
        }
        feed.once('disconnect', () => {
            this.feeds.remove(feed);
        });
    }
    nodeList() {
        return this.nodes.entries;
    }
    broadcast(data) {
        for (const feed of this.feeds.entries) {
            feed.send(data);
        }
    }
    timeoutCheck() {
        const now = Date.now();
        for (const node of this.nodes.entries) {
            node.timeoutCheck(now);
        }
    }
    updateBlock(node) {
        if (node.height > this.height) {
            this.height = node.height;
            this.broadcast(feed_1.default.bestBlock(this.height));
            console.log(`New block ${this.height}`);
        }
        this.broadcast(feed_1.default.imported(node));
        console.log(`${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
    }
}
exports.default = Aggregator;
//# sourceMappingURL=aggregator.js.map