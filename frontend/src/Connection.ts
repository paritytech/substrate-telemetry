import { VERSION, timestamp, FeedMessage, Types, Maybe, sleep } from './common';
import {
  State,
  Update,
  Node,
  UpdateBound,
  ChainData,
  PINNED_CHAINS,
} from './state';
import { PersistentSet } from './persist';
import { getHashData, setHashData } from './utils';
import { AfgHandling } from './AfgHandling';
import { VIS_AUTHORITIES_LIMIT } from './components/Consensus';
import { Column } from './components/List';
import { ACTIONS } from './common/feed';

const TIMEOUT_BASE = (1000 * 5) as Types.Milliseconds; // 5 seconds
const TIMEOUT_MAX = (1000 * 60 * 5) as Types.Milliseconds; // 5 minutes

declare global {
  interface Window {
    process_env: string;
  }
}

export class Connection {
  public static async create(
    pins: PersistentSet<Types.NodeName>,
    update: Update
  ): Promise<Connection> {
    return new Connection(await Connection.socket(), update, pins);
  }

  private static readonly utf8decoder = new TextDecoder('utf-8');
  private static readonly address = Connection.getAddress();

  private static getAddress(): string {
    const ENV_URL = 'SUBSTRATE_TELEMETRY_URL';

    if (process.env && process.env[ENV_URL]) {
      return process.env[ENV_URL] as string;
    }

    if (window.process_env && window.process_env[ENV_URL]) {
      return window.process_env[ENV_URL];
    }

    if (window.location.protocol === 'https:') {
      return `wss://${window.location.hostname}/feed/`;
    }

    return `ws://127.0.0.1:8000/feed`;
  }

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

      socket.binaryType = 'arraybuffer';
      socket.addEventListener('open', onSuccess);
      socket.addEventListener('error', onFailure);
      socket.addEventListener('close', onFailure);
    });
  }

  private pingId = 0;
  private pingTimeout: NodeJS.Timer;
  private pingSent: Maybe<Types.Timestamp> = null;
  private resubscribeTo: Maybe<Types.ChainLabel> = getHashData().chain;
  private resubscribeSendFinality: boolean = getHashData().tab === 'consensus';
  private socket: WebSocket;
  private state: Readonly<State>;
  private readonly update: Update;
  private readonly pins: PersistentSet<Types.NodeName>;
  constructor(
    socket: WebSocket,
    update: Update,
    pins: PersistentSet<Types.NodeName>
  ) {
    this.socket = socket;
    this.update = update;
    this.pins = pins;
    this.bindSocket();
  }

  public subscribe(chain: Types.ChainLabel) {
    if (this.state.subscribed != null && this.state.subscribed !== chain) {
      this.state = this.update({
        tab: 'list',
      });
      setHashData({ chain, tab: 'list' });
    } else {
      setHashData({ chain });
    }

    this.socket.send(`subscribe:${chain}`);
  }

  public subscribeConsensus(chain: Types.ChainLabel) {
    if (this.state.authorities.length <= VIS_AUTHORITIES_LIMIT) {
      setHashData({ chain });
      this.resubscribeSendFinality = true;
      this.socket.send(`send-finality:${chain}`);
    }
  }

  public resetConsensus() {
    this.state = this.update({
      consensusInfo: new Array() as Types.ConsensusInfo,
      displayConsensusLoadingScreen: true,
      authorities: [] as Types.Address[],
      authoritySetId: null,
    });
  }

  public unsubscribeConsensus(chain: Types.ChainLabel) {
    this.resubscribeSendFinality = true;
    this.socket.send(`no-more-finality:${chain}`);
  }

  public handleMessages = (messages: FeedMessage.Message[]) => {
    const { nodes, chains, sortBy, selectedColumns } = this.state;
    const ref = nodes.ref();

    const updateState: UpdateBound = (state) => {
      this.state = this.update(state);
    };
    const getState = () => this.state;
    const afg = new AfgHandling(updateState, getState);

    let sortByColumn: Maybe<Column> = null;

    if (sortBy != null) {
      sortByColumn =
        sortBy < 0 ? selectedColumns[~sortBy] : selectedColumns[sortBy];
    }

    for (const message of messages) {
      switch (message.action) {
        case ACTIONS.FeedVersion: {
          if (message.payload !== VERSION) {
            this.state = this.update({ status: 'upgrade-requested' });
            this.clean();

            // Force reload from the server
            setTimeout(() => window.location.reload(true), 3000);

            return;
          }

          break;
        }

        case ACTIONS.BestBlock: {
          const [best, blockTimestamp, blockAverage] = message.payload;

          nodes.mutEach((node) => node.newBestBlock());

          this.state = this.update({ best, blockTimestamp, blockAverage });

          break;
        }

        case ACTIONS.BestFinalized: {
          const [finalized /*, hash */] = message.payload;

          this.state = this.update({ finalized });

          break;
        }

        case ACTIONS.AddedNode: {
          const [
            id,
            nodeDetails,
            nodeStats,
            nodeIO,
            nodeHardware,
            blockDetails,
            location,
            connectedAt,
          ] = message.payload;
          const pinned = this.pins.has(nodeDetails[0]);
          const node = new Node(
            pinned,
            id,
            nodeDetails,
            nodeStats,
            nodeIO,
            nodeHardware,
            blockDetails,
            location,
            connectedAt
          );

          nodes.add(node);

          break;
        }

        case ACTIONS.RemovedNode: {
          const id = message.payload;

          nodes.remove(id);

          break;
        }

        case ACTIONS.StaleNode: {
          const id = message.payload;

          nodes.mutAndSort(id, (node) => node.setStale(true));

          break;
        }

        case ACTIONS.LocatedNode: {
          const [id, lat, lon, city] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateLocation([lat, lon, city]),
            sortByColumn === Column.LOCATION
          );

          break;
        }

        case ACTIONS.ImportedBlock: {
          const [id, blockDetails] = message.payload;

          nodes.mutAndSort(id, (node) => node.updateBlock(blockDetails));

          break;
        }

        case ACTIONS.FinalizedBlock: {
          const [id, height, hash] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateFinalized(height, hash),
            sortByColumn === Column.FINALIZED ||
              sortByColumn === Column.FINALIZED_HASH
          );

          break;
        }

        case ACTIONS.NodeStats: {
          const [id, nodeStats] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateStats(nodeStats),
            sortByColumn === Column.PEERS || sortByColumn === Column.TXS
          );

          break;
        }

        case ACTIONS.NodeHardware: {
          const [id, nodeHardware] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateHardware(nodeHardware),
            sortByColumn === Column.CPU ||
              sortByColumn === Column.MEM ||
              sortByColumn === Column.UPLOAD ||
              sortByColumn === Column.DOWNLOAD
          );

          break;
        }

        case ACTIONS.NodeIO: {
          const [id, nodeIO] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateIO(nodeIO),
            sortByColumn === Column.STATE_CACHE ||
              sortByColumn === Column.DB_CACHE ||
              sortByColumn === Column.DISK_READ ||
              sortByColumn === Column.DISK_WRITE
          );

          break;
        }

        case ACTIONS.TimeSync: {
          this.state = this.update({
            timeDiff: (timestamp() - message.payload) as Types.Milliseconds,
          });

          break;
        }

        case ACTIONS.AddedChain: {
          const [label, nodeCount] = message.payload;
          const chain = chains.get(label);

          if (chain) {
            chain.nodeCount = nodeCount;
          } else {
            chains.set(label, { label, nodeCount });
          }

          this.state = this.update({ chains });

          break;
        }

        case ACTIONS.RemovedChain: {
          chains.delete(message.payload);

          if (this.state.subscribed === message.payload) {
            nodes.clear();
            this.state = this.update({ subscribed: null, nodes, chains });
            this.resetConsensus();
          }

          break;
        }

        case ACTIONS.SubscribedTo: {
          nodes.clear();

          this.state = this.update({ subscribed: message.payload, nodes });

          break;
        }

        case ACTIONS.UnsubscribedFrom: {
          if (this.state.subscribed === message.payload) {
            nodes.clear();

            this.state = this.update({ subscribed: null, nodes });
          }

          break;
        }

        case ACTIONS.Pong: {
          this.pong(Number(message.payload));

          break;
        }

        case ACTIONS.AfgFinalized: {
          const [nodeAddress, finalizedNumber, finalizedHash] = message.payload;
          const no = parseInt(String(finalizedNumber), 10) as Types.BlockNumber;
          afg.receivedFinalized(nodeAddress, no, finalizedHash);

          break;
        }

        case ACTIONS.AfgReceivedPrevote: {
          const [nodeAddress, blockNumber, blockHash, voter] = message.payload;
          const no = parseInt(String(blockNumber), 10) as Types.BlockNumber;
          afg.receivedPre(nodeAddress, no, voter, 'prevote');

          break;
        }

        case ACTIONS.AfgReceivedPrecommit: {
          const [nodeAddress, blockNumber, blockHash, voter] = message.payload;
          const no = parseInt(String(blockNumber), 10) as Types.BlockNumber;
          afg.receivedPre(nodeAddress, no, voter, 'precommit');

          break;
        }

        case ACTIONS.AfgAuthoritySet: {
          const [authoritySetId, authorities] = message.payload;
          afg.receivedAuthoritySet(authoritySetId, authorities);

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
  };

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
      this.resubscribeSendFinality = this.state.sendFinality;
      this.state = this.update({ subscribed: null, sendFinality: false });
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
  };

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
    const data =
      typeof event.data === 'string'
        ? ((event.data as any) as FeedMessage.Data)
        : ((Connection.utf8decoder.decode(
            event.data
          ) as any) as FeedMessage.Data);

    this.handleMessages(FeedMessage.deserialize(data));
  };

  private autoSubscribe() {
    const { subscribed, chains } = this.state;
    const { resubscribeTo, resubscribeSendFinality } = this;

    if (subscribed) {
      return;
    }

    if (resubscribeTo) {
      if (chains.has(resubscribeTo)) {
        this.subscribe(resubscribeTo);
        if (resubscribeSendFinality) {
          this.subscribeConsensus(resubscribeTo);
        }
        return;
      }
    }

    let topChain: Maybe<ChainData> = null;

    for (const chain of chains.values()) {
      if (PINNED_CHAINS[chain.label] === 1) {
        topChain = chain;
        break;
      }

      if (!topChain || chain.nodeCount > topChain.nodeCount) {
        topChain = chain;
      }
    }

    if (topChain) {
      this.subscribe(topChain.label);
    }
  }

  private handleDisconnect = async () => {
    this.state = this.update({ status: 'offline' });
    this.resetConsensus();
    this.clean();
    this.socket.close();
    this.socket = await Connection.socket();
    this.bindSocket();
  };
}
