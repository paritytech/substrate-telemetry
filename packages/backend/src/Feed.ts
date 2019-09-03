import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import Node from './Node';
import Chain from './Chain';
import Block from './Block';
import { VERSION, timestamp, Maybe, FeedMessage, Types, idGenerator } from '@dotstats/common';
import { Location } from './location';

const nextId = idGenerator<Types.FeedId>();
const { Actions } = FeedMessage;

export default class Feed {
  public id: Types.FeedId;

  public chain: Maybe<Types.ChainLabel> = null;
  public readonly events = new EventEmitter();

  private socket: WebSocket;
  private messages: Array<FeedMessage.Message> = [];
  private waitingForPong = false;
  private sendFinality = false;

  constructor(socket: WebSocket) {
    this.id = nextId();
    this.socket = socket;

    socket.on('message', this.handleCommand);
    socket.on('error', this.disconnect);
    socket.on('close', this.disconnect);
    socket.on('pong', this.onPong);
  }

  public static feedVersion(): FeedMessage.Message {
    return {
      action: Actions.FeedVersion,
      payload: VERSION
    };
  }

  public static bestBlock(height: Types.BlockNumber, ts: Types.Timestamp, avg: Maybe<Types.Milliseconds>): FeedMessage.Message {
    return {
      action: Actions.BestBlock,
      payload: [height, ts, avg]
    };
  }

  public static bestFinalizedBlock(block: Block): FeedMessage.Message {
    return {
      action: Actions.BestFinalized,
      payload: [block.number, block.hash]
    };
  }

  public static addedNode(node: Node): FeedMessage.Message {
    return {
      action: Actions.AddedNode,
      payload: [node.id, node.nodeDetails(), node.nodeStats(), node.nodeHardware(), node.blockDetails(), node.nodeLocation()]
    };
  }

  public static removedNode(node: Node): FeedMessage.Message {
    return {
      action: Actions.RemovedNode,
      payload: node.id
    };
  }

  public static staleNode(node: Node): FeedMessage.Message {
    return {
      action: Actions.StaleNode,
      payload: node.id
    }
  }

  public static locatedNode(node: Node, location: Location): FeedMessage.Message {
    return {
      action: Actions.LocatedNode,
      payload: [node.id, location.lat, location.lon, location.city]
    };
  }

  public static imported(node: Node): FeedMessage.Message {
    return {
      action: Actions.ImportedBlock,
      payload: [node.id, node.blockDetails()]
    };
  }

  public static finalized(node: Node): FeedMessage.Message {
    return {
      action: Actions.FinalizedBlock,
      payload: [node.id, node.finalized.number, node.finalized.hash]
    };
  }

  public static stats(node: Node): FeedMessage.Message {
    return {
      action: Actions.NodeStats,
      payload: [node.id, node.nodeStats()]
    };
  }

  public static afgFinalized(node: Node, finalizedNumber: Types.BlockNumber, finalizedHash: Types.BlockHash): FeedMessage.Message {
    const addr = node.address != null ? node.address : "" as Types.Address;
    return {
      action: Actions.AfgFinalized,
      payload: [addr, finalizedNumber, finalizedHash]
    };
  }

  public static afgReceivedPrevote(
    node: Node,
    targetNumber: Types.BlockNumber,
    targetHash: Types.BlockHash,
    voter: Types.Address
  ): FeedMessage.Message {
    const addr = node.address != null ? node.address : "" as Types.Address;
    return {
      action: Actions.AfgReceivedPrevote,
      payload: [addr, targetNumber, targetHash, voter]
    };
  }

  public static afgReceivedPrecommit(
    node: Node,
    targetNumber: Types.BlockNumber,
    targetHash: Types.BlockHash,
    voter: Types.Address
  ): FeedMessage.Message {
    const addr = node.address != null ? node.address : "" as Types.Address;
    return {
      action: Actions.AfgReceivedPrecommit,
      payload: [addr, targetNumber, targetHash, voter]
    };
  }

  public static afgAuthoritySet(
    authoritySetInfo: Types.AuthoritySetInfo,
  ): FeedMessage.Message {
    return {
      action: Actions.AfgAuthoritySet,
      payload: authoritySetInfo,
    };
  }

  public static hardware(node: Node): FeedMessage.Message {
    return {
      action: Actions.NodeHardware,
      payload: [node.id, node.nodeHardware()]
    };
  }

  public static timeSync(): FeedMessage.Message {
    return {
      action: Actions.TimeSync,
      payload: timestamp()
    };
  }

  public static addedChain(chain: Chain): FeedMessage.Message {
    return {
      action: Actions.AddedChain,
      payload: [chain.label, chain.nodeCount]
    };
  }

  public static removedChain(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.RemovedChain,
      payload: label
    };
  }

  public static subscribedTo(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.SubscribedTo,
      payload: label
    };
  }

  public static unsubscribedFrom(label: Types.ChainLabel): FeedMessage.Message {
    return {
      action: Actions.UnsubscribedFrom,
      payload: label
    };
  }

  public static pong(payload: string): FeedMessage.Message {
    return {
      action: Actions.Pong,
      payload
    };
  }

  public sendData(data: FeedMessage.Data) {
    this.socket.send(data, this.handleError);
  }

  public sendMessage(message: FeedMessage.Message) {
    const queue = this.messages.length === 0;

    this.messages.push(message);

    if (queue) {
      process.nextTick(this.sendMessages);
    }
  }

  public sendConsensusMessage(message: FeedMessage.Message) {
    if (!this.sendFinality) {
      return;
    }

    this.sendMessage(message);
  }

  public ping() {
    if (this.waitingForPong) {
      this.disconnect();
      return;
    }
    this.waitingForPong = true;

    this.socket.ping(this.handleError);
  }

  private sendMessages = () => {
    const data = FeedMessage.serialize(this.messages);
    this.messages = [];
    this.socket.send(data, this.handleError);
  }

  private handleCommand = (data: WebSocket.Data) => {
    const [tag, payload] = data.toString().split(':', 2) as [string, Maybe<string>];

    if (!payload) {
      return;
    }

    switch (tag) {
      case 'subscribe':
        if (this.chain) {
          this.events.emit('unsubscribe', this.chain);
          this.chain = null;
        }

        this.events.emit('subscribe', payload as Types.ChainLabel);
        break;

      case 'send-finality':
        this.sendFinality = true;
        break;

      case 'no-more-finality':
        this.sendFinality = false;
        break;

      case 'ping':
        this.sendMessage(Feed.pong(payload));
        break;

      default:
        console.error('Unknown command tag:', tag);
    }
  }

  private handleError = (err: Maybe<Error>) => {
    if (err) {
      console.error('Error when sending data to the socket', err);

      this.disconnect();
    }
  }

  private disconnect = () => {
    this.socket.removeListener('message', this.handleCommand);
    this.socket.removeListener('error', this.disconnect);
    this.socket.removeListener('close', this.disconnect);
    this.socket.removeListener('pong', this.onPong);
    this.socket.terminate();

    this.events.emit('disconnect');
  }

  private onPong = () => {
    this.waitingForPong = false;
  }
}
