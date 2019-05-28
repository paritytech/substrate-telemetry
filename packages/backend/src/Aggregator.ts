import Chain from './Chain';
import Node from './Node';
import Feed from './Feed';
import FeedSet from './FeedSet';
import { Types, FeedMessage, Maybe, timestamp } from '@dotstats/common';

export default class Aggregator {
  private readonly chains = new Map<Types.ChainLabel, Chain>();
  private readonly feeds = new FeedSet();

  constructor() {
    setInterval(() => this.timeoutCheck(), 10000);
  }

  public addNode(node: Node) {
    let chain = this.getChain(node.chain);

    chain.addNode(node);

    this.feeds.broadcast(Feed.addedChain(chain));
  }

  public addFeed(feed: Feed) {
    this.feeds.add(feed);

    feed.sendMessage(Feed.feedVersion());

    for (const chain of this.chains.values()) {
      feed.sendMessage(Feed.addedChain(chain));
    }

    feed.events.on('subscribe', (label: Types.ChainLabel) => {
      const chain = this.chains.get(label);

      if (chain) {
        feed.sendMessage(Feed.subscribedTo(label));
        chain.addFeed(feed);
      }
    });

    feed.events.on('unsubscribe', (label: Types.ChainLabel) => {
      const chain = this.chains.get(label);

      if (chain) {
        chain.removeFeed(feed);
        feed.sendMessage(Feed.unsubscribedFrom(label));
      }
    });

  }

  public getExistingChain(label: Types.ChainLabel) : Maybe<Chain> {
    return this.chains.get(label);
  }

  private getChain(label: Types.ChainLabel): Chain {
    const chain = this.chains.get(label);

    if (chain) {
      return chain;
    } else {
      const chain = new Chain(label);

      chain.events.on('disconnect', (count: number) => {
        if (count !== 0) {
          this.feeds.broadcast(Feed.addedChain(chain));

          return;
        }

        chain.events.removeAllListeners();

        this.chains.delete(chain.label);

        console.log(`Chain: ${label} lost all nodes`);
        this.feeds.broadcast(Feed.removedChain(label));
      });

      this.chains.set(label, chain);

      console.log(`New chain: ${label}`);
      this.feeds.broadcast(Feed.addedChain(chain));

      return chain;
    }
  }

  private timeoutCheck() {
    const empty: Types.ChainLabel[] = [];

    const now = timestamp();

    for (const chain of this.chains.values()) {
      chain.timeoutCheck(now);
    }

    for (const feed of this.feeds.values()) {
      feed.ping();
    }
  }
}
