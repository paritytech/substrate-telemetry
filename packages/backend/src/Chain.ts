import * as EventEmitter from 'events';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import Block from './Block';
import { Maybe, Types, FeedMessage, NumStats } from '@dotstats/common';
import { BlockNumber, ConsensusInfo } from "@dotstats/common/build/types";

const BLOCK_TIME_HISTORY = 10;
export const MAX_BLOCKS_IN_CHAIN_CACHE = 50;

export default class Chain {
  private nodes = new Set<Node>();
  private lastBroadcastCache: Types.ConsensusInfo = {} as ConsensusInfo;
  private feeds = new FeedSet();

  public readonly events = new EventEmitter();
  public readonly label: Types.ChainLabel;

  public height = 0 as Types.BlockNumber;
  public finalized = Block.ZERO;
  public blockTimestamp = 0 as Types.Timestamp;
  public chainConsensusCache: Types.ConsensusInfo = {} as ConsensusInfo;

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

    node.events.once('disconnect', () => this.removeNode(node));

    node.events.on('block', () => this.updateBlock(node));
    node.events.on('finalized', () => this.updateFinalized(node));
    node.events.on('consensus-info', () => this.updateConsensusInfo(node));
    node.events.on('authority-set-changed', (authorities, authoritySetId, blockNumber, blockHash) =>
        this.authoritySetChanged(node, authorities, authoritySetId));
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

  public addFeed(feed: Feed) {
    this.feeds.add(feed);

    // TODO: this is a bit unclean, find a better way
    feed.chain = this.label;

    feed.sendMessage(Feed.timeSync());
    feed.sendMessage(Feed.bestBlock(this.height, this.blockTimestamp, this.averageBlockTime));
    feed.sendMessage(Feed.bestFinalizedBlock(this.finalized));

    for (const node of this.nodes.values()) {
      feed.sendMessage(Feed.addedNode(node));
      feed.sendMessage(Feed.finalized(node));
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

    this.feeds.broadcast(Feed.imported(node));
    this.updateConsensusInfo(node);

    console.log(`[${this.label}] ${node.name} imported ${height}, block time: ${node.blockTime / 1000}s, average: ${node.average / 1000}s | latency ${node.latency}`);
  }

  private downgradeBlock() {
    let height = 0 as Types.BlockNumber;
    let finalized = Block.ZERO;

    for (const node of this.nodes) {
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

    this.updateConsensusInfo(node);
  }

  private initialiseConsensusView(height: BlockNumber, id_node1: string, addr_node2: string) {
    if (this.chainConsensusCache[height] === undefined) {
      this.chainConsensusCache[height] = {};
    }
    if (this.chainConsensusCache[height][id_node1] === undefined) {
      this.chainConsensusCache[height][id_node1] = {};
      this.chainConsensusCache[height][id_node1][addr_node2] = {} as Types.ConsensusInfo;
    }
    if (this.chainConsensusCache[height][id_node1][addr_node2] === undefined) {
      this.chainConsensusCache[height][id_node1][addr_node2] = {} as Types.ConsensusInfo;
    }
  }

  private updateConsensusInfo(node: Node) {
    for (let height in node.consensusCache) {
      if (height !== undefined) {
        this.initialiseConsensusView(parseInt(height) as BlockNumber, String(node.id), String(node.address));
        this.chainConsensusCache[height][String(node.id)] = node.consensusCache[height];
      }
    }

    // broadcast only the cache blocks which changed
    const delta: Types.ConsensusInfo = {} as ConsensusInfo;
    const keys = Object.keys(this.chainConsensusCache);
    const tip = keys[keys.length - 1];
    for (let height in this.chainConsensusCache) {
      const in_cache_range = parseInt(tip) - parseInt(height) < MAX_BLOCKS_IN_CHAIN_CACHE;
      if (!in_cache_range) {
        continue
      }

      const current = this.chainConsensusCache[height];
      const old = this.lastBroadcastCache[height];

      if (JSON.stringify(current) !== JSON.stringify(old)) {
        delta[height] = current;
        this.lastBroadcastCache[height] = JSON.parse(JSON.stringify(current));
      }
    }

    if (Object.keys(delta).length > 0 ) {
      this.feeds.broadcast(Feed.consensusInfo(delta));
      this.truncateChainConsensusCache();
    }
  }

  private authoritySetChanged(node: Node, authorities: Types.Authorities, authoritySetId: Types.AuthoritySetId) {
    if (node.isAuthority()) {
      this.feeds.broadcast(Feed.authoritySet(authorities, authoritySetId));

      if (authoritySetId > 0) {
        this.restartVis();
      }
    }
  }

  private truncateChainConsensusCache() {
    let list = Object.keys(this.chainConsensusCache).reverse();
    list.map((k, i) => {
      if (i > MAX_BLOCKS_IN_CHAIN_CACHE) {
        delete this.chainConsensusCache[k];
      }
    });
  }

  private restartVis() {
    this.nodes.forEach(node => node.resetCache());
    this.feeds.broadcast(Feed.consensusInfo(this.chainConsensusCache));
    this.chainConsensusCache = {} as ConsensusInfo;
  }

  private updateAverageBlockTime(height: Types.BlockNumber, now: Types.Timestamp) {
    this.blockTimes.push((now - this.blockTimestamp) as Types.Milliseconds);

    // We are guaranteed that count > 0
    this.averageBlockTime = this.blockTimes.average();
  }
}
