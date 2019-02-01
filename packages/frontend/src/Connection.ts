import { VERSION, timestamp, FeedMessage, Types, Maybe, sleep } from '@dotstats/common';
import { State, Update, Node } from './state';
import { PersistentSet } from './persist';
import { getHashData, setHashData } from './utils';

const { Actions } = FeedMessage;

const TIMEOUT_BASE = (1000 * 5) as Types.Milliseconds; // 5 seconds
const TIMEOUT_MAX = (1000 * 60 * 5) as Types.Milliseconds; // 5 minutes

export class Connection {
  public static async create(pins: PersistentSet<Types.NodeName>, update: Update): Promise<Connection> {
    return new Connection(await Connection.socket(), update, pins);
  }

  private static readonly address = window.location.protocol === 'https:'
                                      ? `wss://${window.location.hostname}/feed/`
                                      : `ws://${window.location.hostname}:8080`;

  // private static readonly address = 'wss://telemetry.polkadot.io/feed/';

  private static async socket(): Promise<WebSocket> {
    let socket = await Connection.trySocket();
    let timeout = TIMEOUT_BASE;

    while (!socket) {
      await sleep(timeout);

      timeout = Math.min(timeout * 2, TIMEOUT_MAX) as Types.Milliseconds;
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

  private pingId = 0;
  private pingTimeout: NodeJS.Timer;
  private pingSent: Maybe<Types.Timestamp> = null;
  private resubscribeTo: Maybe<Types.ChainLabel> = getHashData().chain;
  private socket: WebSocket;
  private state: Readonly<State>;
  private readonly update: Update;
  private readonly pins: PersistentSet<Types.NodeName>;

  constructor(socket: WebSocket, update: Update, pins: PersistentSet<Types.NodeName>) {
    this.socket = socket;
    this.update = update;
    this.pins = pins;
    this.bindSocket();
  }

  public subscribe(chain: Types.ChainLabel) {
    setHashData({ chain });
    this.socket.send(`subscribe:${chain}`);
  }

  public handleMessages = (messages: FeedMessage.Message[]) => {
    const { nodes, chains } = this.state;
    const ref = nodes.ref();

    for (const message of messages) {
      switch (message.action) {
        case Actions.FeedVersion: {
          if (message.payload !== VERSION) {
            this.state = this.update({ status: 'upgrade-requested' });
            this.clean();

            // Force reload from the server
            setTimeout(() => window.location.reload(true), 3000);

            return;
          }

          break;
        }

        case Actions.BestBlock: {
          const [best, blockTimestamp, blockAverage] = message.payload;

          nodes.mutEach((node) => node.newBestBlock());

          this.state = this.update({ best, blockTimestamp, blockAverage });

          break;
        }

        case Actions.BestFinalized: {
          const [finalized /*, hash */] = message.payload;

          this.state = this.update({ finalized });

          break;
        }

        case Actions.AddedNode: {
          const [id, nodeDetails, nodeStats, nodeHardware, blockDetails, location] = message.payload;
          const pinned = this.pins.has(nodeDetails[0]);
          const node = new Node(pinned, id, nodeDetails, nodeStats, nodeHardware, blockDetails, location);

          nodes.add(node);

          break;
        }

        case Actions.RemovedNode: {
          const id = message.payload;

          nodes.remove(id);

          break;
        }

        case Actions.LocatedNode: {
          const [id, lat, lon, city] = message.payload;

          nodes.mut(id, (node) => node.updateLocation([lat, lon, city]));

          break;
        }

        case Actions.ImportedBlock: {
          const [id, blockDetails] = message.payload;

          nodes.mutAndSort(id, (node) => node.updateBlock(blockDetails));

          break;
        }

        case Actions.FinalizedBlock: {
          const [id, height, hash] = message.payload;

          nodes.mut(id, (node) => node.updateFinalized(height, hash));

          break;
        }

        case Actions.NodeStats: {
          const [id, nodeStats] = message.payload;

          nodes.mut(id, (node) => node.updateStats(nodeStats));

          break;
        }

        case Actions.NodeHardware: {
          const [id, nodeHardware] = message.payload;

          nodes.mut(id, (node) => node.updateHardware(nodeHardware));

          break;
        }

        case Actions.TimeSync: {
          this.state = this.update({
            timeDiff: (timestamp() - message.payload) as Types.Milliseconds
          });

          break;
        }

        case Actions.AddedChain: {
          const [label, nodeCount] = message.payload;
          chains.set(label, nodeCount);

          this.state = this.update({ chains });

          break;
        }

        case Actions.RemovedChain: {
          chains.delete(message.payload);

          if (this.state.subscribed === message.payload) {
            nodes.clear();
            this.state = this.update({ subscribed: null, nodes, chains });
          }

          break;
        }

        case Actions.SubscribedTo: {
          nodes.clear();

          this.state = this.update({ subscribed: message.payload, nodes });

          break;
        }

        case Actions.UnsubscribedFrom: {
          if (this.state.subscribed === message.payload) {
            nodes.clear();

            this.state = this.update({ subscribed: null, nodes });
          }

          break;
        }

        case Actions.Pong: {
          this.pong(Number(message.payload));

          break;
        }

        default: {
          break;
        }
      }
    }

    if (nodes.hasChangedSince(ref)) {
      this.state = this.update({ nodes });
    }

    this.autoSubscribe();
  }

  private bindSocket() {
    this.ping();

    if (this.state) {
      const { nodes } = this.state;
      nodes.clear();
    }

    this.state = this.update({
      status: 'online',
    });

    if (this.state.subscribed) {
      this.resubscribeTo = this.state.subscribed;
      this.state = this.update({ subscribed: null });
    }

    this.socket.addEventListener('message', this.handleFeedData);
    this.socket.addEventListener('close', this.handleDisconnect);
    this.socket.addEventListener('error', this.handleDisconnect);
  }

  private ping = () => {
    if (this.pingSent) {
      this.handleDisconnect();
      return;
    }

    this.pingId += 1;
    this.pingSent = timestamp();
    this.socket.send(`ping:${this.pingId}`);

    this.pingTimeout = setTimeout(this.ping, 30000);
  }

  private pong(id: number) {
    if (!this.pingSent) {
      console.error('Received a pong without sending a ping first');

      this.handleDisconnect();
      return;
    }

    if (id !== this.pingId) {
      console.error('pingId differs');

      this.handleDisconnect();
    }

    const latency = timestamp() - this.pingSent;
    this.pingSent = null;

    console.log('latency', latency);
  }

  private clean() {
    clearTimeout(this.pingTimeout);
    this.pingSent = null;

    this.socket.removeEventListener('message', this.handleFeedData);
    this.socket.removeEventListener('close', this.handleDisconnect);
    this.socket.removeEventListener('error', this.handleDisconnect);
  }

  private handleFeedData = (event: MessageEvent) => {
    const data = event.data as FeedMessage.Data;

    this.handleMessages(FeedMessage.deserialize(data));
  }

  private autoSubscribe() {
    const { subscribed, chains } = this.state;
    const { resubscribeTo } = this;

    if (subscribed) {
      return;
    }

    if (resubscribeTo) {
      if (chains.has(resubscribeTo)) {
        this.subscribe(resubscribeTo);
        return;
      }
    }

    let topLabel: Maybe<Types.ChainLabel> = null;
    let topCount: Types.NodeCount = 0 as Types.NodeCount;

    for (const [label, count] of chains.entries()) {
      if (count > topCount) {
        topLabel = label;
        topCount = count;
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
