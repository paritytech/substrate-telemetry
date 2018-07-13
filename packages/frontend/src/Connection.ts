import { VERSION, timestamp, FeedMessage, Types, Maybe, sleep } from '@dotstats/common';
import { State, Update } from './state';

const { Actions } = FeedMessage;

const TIMEOUT_BASE = (1000 * 5) as Types.Milliseconds; // 5 seconds
const TIMEOUT_MAX = (1000 * 60 * 5) as Types.Milliseconds; // 5 minutes

export class Connection {
  public static async create(update: Update): Promise<Connection> {
    return new Connection(await Connection.socket(), update);
  }

  private static readonly address = `ws://${window.location.hostname}:8080`;

  private static async socket(): Promise<WebSocket> {
    let socket = await Connection.trySocket();
    let timeout = TIMEOUT_BASE;

    while (!socket) {
      await sleep(timeout);

      timeout = Math.max(timeout * 2, TIMEOUT_MAX) as Types.Milliseconds;
      socket = await Connection.trySocket();
    }

    return socket;
  }

  private static async trySocket(): Promise<Maybe<WebSocket>> {
    return new Promise<Maybe<WebSocket>>((resolve, _) => {
      function clean() {
        socket.removeEventListener('open', onSuccess);
        socket.removeEventListener('close', onFailure);
        socket.removeEventListener('error', onFailure);
      }

      function onSuccess() {
        clean();
        resolve(socket);
      }

      function onFailure() {
        clean();
        resolve(null);
      }

      const socket = new WebSocket(Connection.address);

      socket.addEventListener('open', onSuccess);
      socket.addEventListener('error', onFailure);
      socket.addEventListener('close', onFailure);
    });
  }

  private socket: WebSocket;
  private state: Readonly<State>;
  private readonly update: Update;

  constructor(socket: WebSocket, update: Update) {
    this.socket = socket;
    this.update = update;
    this.bindSocket();
  }

  public subscribe(chain: Types.ChainLabel) {
    this.socket.send(`subscribe:${chain}`);
  }

  private bindSocket() {
    this.state = this.update({
      status: 'online',
      nodes: new Map()
    });
    this.socket.addEventListener('message', this.handleMessages);
    this.socket.addEventListener('close', this.handleDisconnect);
    this.socket.addEventListener('error', this.handleDisconnect);
  }

  private clean() {
    this.socket.removeEventListener('message', this.handleMessages);
    this.socket.removeEventListener('close', this.handleDisconnect);
    this.socket.removeEventListener('error', this.handleDisconnect);
  }

  private handleMessages = (event: MessageEvent) => {
    const data = event.data as FeedMessage.Data;
    const nodes = this.state.nodes;
    const chains = this.state.chains;
    const changes = { nodes, chains };

    messages: for (const message of FeedMessage.deserialize(data)) {
      switch (message.action) {
        case Actions.FeedVersion: {
          if (message.payload !== VERSION) {
            this.state = this.update({ status: 'upgrade-requested' });
            this.clean();

            // Force reload from the server
            setTimeout(() => window.location.reload(true), 3000);

            return;
          }

          continue messages;
        }

        case Actions.BestBlock: {
          const [best, blockTimestamp, blockAverage] = message.payload;

          this.state = this.update({ best, blockTimestamp, blockAverage });

          continue messages;
        }

        case Actions.AddedNode: {
          const [id, nodeDetails, nodeStats, blockDetails] = message.payload;
          const node = { id, nodeDetails, nodeStats, blockDetails };

          nodes.set(id, node);

          break;
        }

        case Actions.RemovedNode: {
          nodes.delete(message.payload);

          break;
        }

        case Actions.ImportedBlock: {
          const [id, blockDetails] = message.payload;
          const node = nodes.get(id);

          if (!node) {
            return;
          }

          node.blockDetails = blockDetails;

          break;
        }

        case Actions.NodeStats: {
          const [id, nodeStats] = message.payload;
          const node = nodes.get(id);

          if (!node) {
            return;
          }

          node.nodeStats = nodeStats;

          break;
        }

        case Actions.TimeSync: {
          this.state = this.update({
            timeDiff: (timestamp() - message.payload) as Types.Milliseconds
          });

          continue messages;
        }

        case Actions.AddedChain: {
          const [label, nodeCount] = message.payload;
          chains.set(label, nodeCount);
          this.autoSubscribe();

          break;
        }

        case Actions.RemovedChain: {
          chains.delete(message.payload);

          if (this.state.subscribed === message.payload) {
            nodes.clear();

            this.state = this.update({ subscribed: null, nodes, chains });
            this.autoSubscribe();

            continue messages;
          }

          break;
        }

        case Actions.SubscribedTo: {
          this.state = this.update({ subscribed: message.payload });

          continue messages;
        }

        case Actions.UnsubscribedFrom: {
          if (this.state.subscribed === message.payload) {
            nodes.clear();
            this.state = this.update({ subscribed: null, nodes });
          }

          continue messages;
        }

        default: {
          continue messages;
        }
      }
    }

    this.state = this.update(changes);
  }

  private autoSubscribe() {
    const { subscribed, chains } = this.state;

    if (subscribed) {
      return;
    }

    let topLabel: Maybe<Types.ChainLabel> = null;
    let topCount: Types.NodeCount = 0 as Types.NodeCount;

    for (const [label, count] of chains.entries()) {
      if (count > topCount) {
        topLabel = label;
        topCount = topCount;
      }
    }

    if (topLabel) {
      this.subscribe(topLabel);
    }
  }

  private handleDisconnect = async () => {
    this.state = this.update({ status: 'offline' });
    this.clean();
    this.socket.close();
    this.socket = await Connection.socket();
    this.bindSocket();
  }
}
