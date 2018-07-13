import * as EventEmitter from 'events';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import { timestamp, Maybe, Types, FeedMessage } from '@dotstats/common';

const BLOCK_TIME_HISTORY = 10;

export default class Chain {
  private nodes = new Set<Node>();
  private feeds = new FeedSet();

  public readonly events = new EventEmitter();
  public readonly label: Types.ChainLabel;

  public height = 0 as Types.BlockNumber;
  public blockTimestamp = 0 as Types.Timestamp;

  private blockTimes: Array<number> = new Array(BLOCK_TIME_HISTORY);

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

  public timeoutCheck() {
    const now = timestamp();

    for (const node of this.nodes.values()) {
      node.timeoutCheck(now);
    }

    this.feeds.broadcast(Feed.timeSync());
  }

  private updateBlock(node: Node) {
    if (node.height > this.height) {
      const { height, blockTimestamp } = node;

      if (this.blockTimestamp) {
        this.blockTimes[height * BLOCK_TIME_HISTORY] = blockTimestamp - this.blockTimestamp;
      }

      this.height = height;
      this.blockTimestamp = blockTimestamp;
      node.propagationTime = 0 as Types.PropagationTime;

      this.feeds.broadcast(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));

      console.log(`[${this.label}] New block ${this.height}`);
    } else if (node.height === this.height) {
      node.propagationTime = (node.blockTimestamp - this.blockTimestamp) as Types.PropagationTime;
    }

    this.feeds.broadcast(Feed.imported(node));

    console.log(`[${this.label}] ${node.name} imported ${node.height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
  }

  private get averageBlockTime(): Maybe<Types.Milliseconds> {
    let sum = 0;
    let count = 0;

    for (const time of this.blockTimes) {
      if (time != null) {
        sum += time;
        count += 1;
      }
    }

    if (count === 0) {
      return null;
    }

    return (sum / count) as Types.Milliseconds;
  }
}
