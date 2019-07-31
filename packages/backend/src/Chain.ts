import * as EventEmitter from 'events';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import Block from './Block';
import { Maybe, Types, NumStats } from '@dotstats/common';

const BLOCK_TIME_HISTORY = 10;

export default class Chain {
  private nodes = new Set<Node>();
  private feeds = new FeedSet();
  private count = 0;

  public readonly events = new EventEmitter();
  public readonly label: Types.ChainLabel;

  public height = 0 as Types.BlockNumber;
  public finalized = Block.ZERO;
  public blockTimestamp = 0 as Types.Timestamp;

  private blockTimes = new NumStats<Types.Milliseconds>(BLOCK_TIME_HISTORY);
  private averageBlockTime: Maybe<Types.Milliseconds> = null;

  public lastBroadcastedAuthoritySetInfo: Maybe<Types.AuthoritySetInfo> = null;

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

    node.events.once('disconnect', () => this.removeNode(node));
    node.events.once('stale', () => this.staleNode(node));

    node.events.on('block', () => this.updateBlock(node));
    node.events.on('finalized', () => this.updateFinalized(node));

    node.events.on('afg-finalized', (finalizedNumber, finalizedHash) => this.feeds.each(
      f => f.sendConsensusMessage(Feed.afgFinalized(node, finalizedNumber, finalizedHash))
    ));
    node.events.on('afg-received-prevote', (finalizedNumber, finalizedHash, voter) => this.feeds.each(
      f => f.sendConsensusMessage(Feed.afgReceivedPrevote(node, finalizedNumber, finalizedHash, voter))
    ));
    node.events.on('afg-received-precommit', (finalizedNumber, finalizedHash, voter) => this.feeds.each(
      f => f.sendConsensusMessage(Feed.afgReceivedPrecommit(node, finalizedNumber, finalizedHash, voter))
    ));
    node.events.on('authority-set-changed', (authorities, authoritySetId, blockNumber, blockHash) => {
      let newSet;
      if (this.lastBroadcastedAuthoritySetInfo == null) {
        newSet = true;
      } else {
        const [lastBroadcastedAuthoritySetId] = this.lastBroadcastedAuthoritySetInfo;
        newSet = authoritySetId !== lastBroadcastedAuthoritySetId;
      }

      if (node.isAuthority() && newSet) {
        const addr = node.address != null ? node.address : "" as Types.Address;
        const set = [authoritySetId, authorities, addr, blockNumber, blockHash] as Types.AuthoritySetInfo;
        this.feeds.broadcast(Feed.afgAuthoritySet(set));
        this.lastBroadcastedAuthoritySetInfo = set;
      }
    });

    node.events.on('stats', () => this.feeds.broadcast(Feed.stats(node)));
    node.events.on('hardware', () => this.feeds.broadcast(Feed.hardware(node)));
    node.events.on('location', (location) => this.feeds.broadcast(Feed.locatedNode(node, location)));

    this.updateBlock(node);
    this.updateFinalized(node);
  }

  public removeNode(node: Node) {
    node.events.removeAllListeners();

    this.nodes.delete(node);
    this.feeds.broadcast(Feed.removedNode(node));
    this.events.emit('disconnect', this.nodeCount);

    if (this.height === node.best.number) {
      this.downgradeBlock();
    }
  }

  public staleNode(node: Node) {
    node.isStale = true;

    this.feeds.broadcast(Feed.staleNode(node));

    if (this.height === node.best.number) {
      this.downgradeBlock();
    }
  }

  public addFeed(feed: Feed) {
    this.feeds.add(feed);

    // TODO: this is a bit unclean, find a better way
    feed.chain = this.label;

    feed.sendMessage(Feed.timeSync());
    feed.sendMessage(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));
    feed.sendMessage(Feed.bestFinalizedBlock(this.finalized));

    if (this.lastBroadcastedAuthoritySetInfo != null) {
      feed.sendMessage(Feed.afgAuthoritySet(this.lastBroadcastedAuthoritySetInfo));
    }

    for (const node of this.nodes.values()) {
      feed.sendMessage(Feed.addedNode(node));
      feed.sendMessage(Feed.finalized(node));

      if (node.isStale) {
        feed.sendMessage(Feed.staleNode(node));
      }
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
    const height = node.best.number;

    if (height > this.height) {
      // New best block
      const { blockTimestamp } = node;

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
    } else if (height === this.height) {
      // Caught up to best block
      node.propagationTime = (node.blockTimestamp - this.blockTimestamp) as Types.PropagationTime;
    }

    if (node.isStale) {
      node.isStale = false;
    }

    this.feeds.broadcast(Feed.imported(node));

    console.log(`[${this.label}] ${node.name} imported ${height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
  }

  private downgradeBlock() {
    let height = 0 as Types.BlockNumber;
    let finalized = Block.ZERO;

    for (const node of this.nodes) {
      if (node.isStale) {
        continue;
      }

      if (this.height === node.best.number) {
        return;
      }

      if (node.best.number > height) {
        height = node.best.number;
      }

      if (node.finalized.number > finalized.number) {
        finalized = node.finalized;
      }
    }

    this.height = height;
    this.finalized = finalized;
    this.feeds.broadcast(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));
    this.feeds.broadcast(Feed.bestFinalizedBlock(this.finalized));
  }

  private updateFinalized(node: Node) {
    if (node.finalized.gt(this.finalized)) {
      this.finalized = node.finalized;

      this.feeds.broadcast(Feed.bestFinalizedBlock(this.finalized));
    }

    this.feeds.broadcast(Feed.finalized(node));
  }

  private updateAverageBlockTime(height: Types.BlockNumber, now: Types.Timestamp) {
    this.blockTimes.push((now - this.blockTimestamp) as Types.Milliseconds);

    // We are guaranteed that count > 0
    this.averageBlockTime = this.blockTimes.average();
  }
}
