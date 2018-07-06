import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import Node from './Node';
import { timestamp, Maybe, FeedMessage, Types, idGenerator } from '@dotstats/common';

const nextId = idGenerator<Types.FeedId>();
const { Actions } = FeedMessage;

export default class Feed {
  public id: Types.FeedId;

  public chain: Maybe<Types.ChainLabel> = null;
  public readonly events = new EventEmitter();

  private socket: WebSocket;
  private messages: Array<FeedMessage.Message> = [];

  constructor(socket: WebSocket) {
    this.id = nextId();
    this.socket = socket;

    socket.on('message', (data) => this.handleCommand(data.toString()));
    socket.on('error', () => this.disconnect());
    socket.on('close', () => this.disconnect());
  }

  public static bestBlock(height: Types.BlockNumber, ts: Types.Timestamp): FeedMessage.Message {
    return {
      action: Actions.BestBlock,
      payload: [height, ts]
    };
  }

  public static addedNode(node: Node): FeedMessage.Message {
    return {
      action: Actions.AddedNode,
      payload: [node.id, node.nodeDetails(), node.nodeStats(), node.blockDetails()]
    };
  }

  public static removedNode(node: Node): FeedMessage.Message {
    return {
      action: Actions.RemovedNode,
      payload: node.id
    };
  }

  public static imported(node: Node): FeedMessage.Message {
    return {
      action: Actions.ImportedBlock,
      payload: [node.id, node.blockDetails()]
    };
  }

  public static stats(node: Node): FeedMessage.Message {
    return {
      action: Actions.NodeStats,
      payload: [node.id, node.nodeStats()]
    };
  }

  public static timeSync(): FeedMessage.Message {
    return {
      action: Actions.TimeSync,
      payload: timestamp()
    };
  }

  public static addedChain(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.AddedChain,
      payload: label
    };
  }

  public static removedChain(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.RemovedChain,
      payload: label
    }
  }

  public static subscribedTo(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.SubscribedTo,
      payload: label,
    }
  }

  public static unsubscribedFrom(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.UnsubscribedFrom,
      payload: label,
    }
  }

  public sendData(data: FeedMessage.Data) {
    this.socket.send(data);
  }

  public sendMessage(message: FeedMessage.Message) {
    const queue = this.messages.length === 0;

    this.messages.push(message);

    if (queue) {
      process.nextTick(this.sendMessages);
    }
  }

  private sendMessages = () => {
    const data = FeedMessage.serialize(this.messages);
    this.messages = [];
    this.socket.send(data);
  }

  private handleCommand(cmd: string) {
    if (cmd.startsWith('subscribe:')) {
      if (this.chain) {
        this.events.emit('unsubscribe', this.chain);
        this.chain = null;
      }

      const label = cmd.substr(10) as Types.ChainLabel;

      this.events.emit('subscribe', label);
    }
  }

  private disconnect() {
    this.socket.removeAllListeners();
    this.socket.close();

    this.events.emit('disconnect');
  }
}
