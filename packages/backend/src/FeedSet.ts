import Feed from './Feed';
import { FeedMessage } from '@dotstats/common';

type DisconnectListener = () => void;

export default class FeedSet {
  private feeds = new Map<Feed, DisconnectListener>();
  private messages: Array<FeedMessage.Message> = [];

  public values(): IterableIterator<Feed> {
    return this.feeds.keys();
  }

  public each(fn: (feed: Feed) => void) {
    for (const feed of this.values()) {
      fn(feed);
    }
  }

  public add(feed: Feed) {
    const listener = () => this.remove(feed);

    this.feeds.set(feed, listener);

    feed.events.once('disconnect', listener);
  }

  public remove(feed: Feed) {
    const listener = this.feeds.get(feed);

    if (!listener) {
      return;
    }

    feed.events.removeListener('disconnect', listener);

    this.feeds.delete(feed);
  }

  public broadcast(message: FeedMessage.Message) {
    const queue = this.messages.length === 0;

    this.messages.push(message);

    if (queue) {
      process.nextTick(this.sendMessages);
    }
  }

  private sendMessages = () => {
    const data = FeedMessage.serialize(this.messages);
    this.messages = [];

    this.each(feed => {
      try {
        feed.sendData(data);
      } catch (err) {
        console.error("Failed to broadcast to feed", err);
      }
    });
  }
}
