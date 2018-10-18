import * as EventEmitter from 'events';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import { Maybe, Types, FeedMessage, NumStats } from '@dotstats/common';

const BLOCK_TIME_HISTORY = 10;

export default class Chain {
  private nodes = new Set<Node>();
  private feeds = new FeedSet();

  public readonly events = new EventEmitter();
  public readonly label: Types.ChainLabel;

  public height = 0 as Types.BlockNumber;
  public blockTimestamp = 0 as Types.Timestamp;

  private blockTimes = new NumStats<Types.Milliseconds>(BLOCK_TIME_HISTORY);
  private averageBlockTime: Maybe<Types.Milliseconds> = null;

  constructor(label: Types.ChainLabel) {
    this.label = label;
  }

  public get nodeCount(): Types.NodeCount {
    return this.nodes.size as Types.NodeCount;
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
    node.events.on('hardware', () => this.feeds.broadcast(Feed.hardware(node)));
    node.events.on('location', (location) => this.feeds.broadcast(Feed.locatedNode(node, location)));

    this.updateBlock(node);
  }

  public addFeed(feed: Feed) {
    this.feeds.add(feed);

    // TODO: this is a bit unclean, find a better way
    feed.chain = this.label;

    feed.sendMessage(Feed.timeSync());
    feed.sendMessage(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));

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

  public timeoutCheck(now: Types.Timestamp) {
    for (const node of this.nodes.values()) {
      node.timeoutCheck(now);
    }

    this.feeds.broadcast(Feed.timeSync());
  }

  private updateBlock(node: Node) {
    if (node.height > this.height) {
      // New best block
      const { height, blockTimestamp } = node;

      if (this.blockTimestamp) {
        this.updateAverageBlockTime(height, blockTimestamp);
      }

      for (const otherNode of this.nodes) {
        otherNode.propagationTime = null;
      }

      this.height = height;
      this.blockTimestamp = blockTimestamp;
      node.propagationTime = 0 as Types.PropagationTime;

      this.feeds.broadcast(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));

      console.log(`[${this.label}] New block ${this.height}`);
    } else if (node.height === this.height) {
      // Caught up to best block
      node.propagationTime = (node.blockTimestamp - this.blockTimestamp) as Types.PropagationTime;
    }

    this.feeds.broadcast(Feed.imported(node));

    console.log(`[${this.label}] ${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
  }

  private updateAverageBlockTime(height: Types.BlockNumber, now: Types.Timestamp) {
    this.blockTimes.push((now - this.blockTimestamp) as Types.Milliseconds);

    // We are guaranteed that count > 0
    this.averageBlockTime = this.blockTimes.average();
  }
}
