// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import { VERSION, timestamp, FeedMessage, Types, Maybe, sleep } from './common';
import { State, Update, Node, ChainData, PINNED_CHAINS } from './state';
import { PersistentSet } from './persist';
import { getHashData, setHashData } from './utils';
import { ACTIONS } from './common/feed';
import {
  Column,
  LocationColumn,
  PeersColumn,
  TxsColumn,
  FinalizedBlockColumn,
  FinalizedHashColumn,
  UploadColumn,
  DownloadColumn,
  StateCacheColumn,
} from './components/List';

const CONNECTION_TIMEOUT_BASE = (1000 * 5) as Types.Milliseconds; // 5 seconds
const CONNECTION_TIMEOUT_MAX = (1000 * 60 * 5) as Types.Milliseconds; // 5 minutes
const MESSAGE_TIMEOUT = (1000 * 60) as Types.Milliseconds; // 60 seconds

declare global {
  interface Window {
    process_env: string;
  }
}

export class Connection {
  public static async create(
    pins: PersistentSet<Types.NodeName>,
    appState: Readonly<State>,
    appUpdate: Update
  ): Promise<Connection> {
    return new Connection(await Connection.socket(), appState, appUpdate, pins);
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

    return 'ws://127.0.0.1:8000/feed';
  }

  private static async socket(): Promise<WebSocket> {
    let socket = await Connection.trySocket();
    let timeout = CONNECTION_TIMEOUT_BASE;

    while (!socket) {
      await sleep(timeout);

      timeout = Math.min(
        timeout * 2,
        CONNECTION_TIMEOUT_MAX
      ) as Types.Milliseconds;
      socket = await Connection.trySocket();
    }

    return socket;
  }

  private static async trySocket(): Promise<Maybe<WebSocket>> {
    return new Promise<Maybe<WebSocket>>((resolve) => {
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

  // timer which will force a reconnection if no message is seen for a while
  private messageTimeout: Maybe<ResettableTimeout> = null;
  // id sent to the backend used to pair responses
  private pingId = 0;
  // timeout handler for ping messages
  private pingTimeout: NodeJS.Timer;
  // timestamp at which the last ping has been sent
  private pingSent: Maybe<Types.Timestamp> = null;
  // chain label to resubsribe to on reconnect
  private resubscribeTo: Maybe<Types.GenesisHash> = getHashData().chain;

  constructor(
    private socket: WebSocket,
    private readonly appState: Readonly<State>,
    private readonly appUpdate: Update,
    private readonly pins: PersistentSet<Types.NodeName>
  ) {
    this.bindSocket();
  }

  public subscribe(chain: Types.GenesisHash) {
    if (
      this.appState.subscribed != null &&
      this.appState.subscribed !== chain
    ) {
      this.appUpdate({
        tab: 'list',
      });
      setHashData({ chain, tab: 'list' });
    } else {
      setHashData({ chain });
    }

    this.socket.send(`subscribe:${chain}`);
  }

  private handleMessages = (messages: FeedMessage.Message[]) => {
    this.messageTimeout?.reset();
    const { nodes, chains, sortBy, selectedColumns } = this.appState;
    const nodesStateRef = nodes.ref;

    let sortByColumn: Maybe<Column> = null;

    if (sortBy != null) {
      sortByColumn =
        sortBy < 0 ? selectedColumns[~sortBy] : selectedColumns[sortBy];
    }

    for (const message of messages) {
      console.log(message);
      switch (message.action) {
        case ACTIONS.FeedVersion: {
          if (message.payload !== VERSION) {
            return this.newVersion();
          }

          break;
        }

        case ACTIONS.BestBlock: {
          const [best, blockTimestamp, blockAverage] = message.payload;

          nodes.mutEach((node) => node.newBestBlock());

          this.appUpdate({ best, blockTimestamp, blockAverage });

          break;
        }

        case ACTIONS.BestFinalized: {
          const [finalized /*, hash */] = message.payload;

          this.appUpdate({ finalized });

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
            startupTime,
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
            startupTime
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
            sortByColumn === LocationColumn
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
            sortByColumn === FinalizedBlockColumn ||
              sortByColumn === FinalizedHashColumn
          );

          break;
        }

        case ACTIONS.NodeStats: {
          const [id, nodeStats] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateStats(nodeStats),
            sortByColumn === PeersColumn || sortByColumn === TxsColumn
          );

          break;
        }

        case ACTIONS.NodeHardware: {
          const [id, nodeHardware] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateHardware(nodeHardware),
            sortByColumn === UploadColumn || sortByColumn === DownloadColumn
          );

          break;
        }

        case ACTIONS.NodeIO: {
          const [id, nodeIO] = message.payload;

          nodes.mutAndMaybeSort(
            id,
            (node) => node.updateIO(nodeIO),
            sortByColumn === StateCacheColumn
          );

          break;
        }

        case ACTIONS.TimeSync: {
          this.appUpdate({
            timeDiff: (timestamp() - message.payload) as Types.Milliseconds,
          });

          break;
        }

        case ACTIONS.AddedChain: {
          const [label, genesisHash, nodeCount] = message.payload;
          const chain = chains.get(genesisHash);

          if (chain) {
            chain.nodeCount = nodeCount;
          } else {
            chains.set(genesisHash, { label, genesisHash, nodeCount });
          }

          this.appUpdate({ chains });

          break;
        }

        case ACTIONS.RemovedChain: {
          chains.delete(message.payload);

          if (this.appState.subscribed === message.payload) {
            nodes.clear();
            this.appUpdate({ subscribed: null, nodes, chains });
          }

          break;
        }

        case ACTIONS.SubscribedTo: {
          nodes.clear();

          this.appUpdate({ subscribed: message.payload, nodes });

          break;
        }

        case ACTIONS.UnsubscribedFrom: {
          if (this.appState.subscribed === message.payload) {
            nodes.clear();

            this.appUpdate({ subscribed: null, nodes });
          }

          break;
        }

        case ACTIONS.Pong: {
          this.pong(Number(message.payload));

          break;
        }

        case ACTIONS.ChainStatsUpdate: {
          this.appUpdate({ chainStats: message.payload });
          break;
        }

        default: {
          break;
        }
      }
    }

    if (nodes.hasChangedSince(nodesStateRef)) {
      this.appUpdate({ nodes });
    }

    this.autoSubscribe();
  };

  private bindSocket() {
    console.log('Connected');
    // Disconnect if no messages are received in 60s:
    this.messageTimeout = resettableTimeout(
      this.handleDisconnect,
      MESSAGE_TIMEOUT
    );
    // Ping periodically to keep the above happy even if no other data is coming in:
    this.ping();

    if (this.appState) {
      const { nodes } = this.appState;
      nodes.clear();
    }

    this.appUpdate({
      status: 'online',
    });

    if (this.appState.subscribed) {
      this.resubscribeTo = this.appState.subscribed;
      this.appUpdate({ subscribed: null });
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
    console.log(`Ping latency: ${latency}ms`);

    this.pingSent = null;

    // Schedule a new ping to be sent at least 30s after the last one:
    this.pingTimeout = setTimeout(
      this.ping,
      Math.max(0, MESSAGE_TIMEOUT / 2 - latency)
    );
  }

  private newVersion() {
    this.appUpdate({ status: 'upgrade-requested' });
    this.clean();

    // Force reload from the server
    setTimeout(() => window.location.reload(), 3000);
  }

  private clean() {
    clearTimeout(this.pingTimeout);
    this.pingSent = null;
    this.messageTimeout?.cancel();
    this.messageTimeout = null;

    this.socket.removeEventListener('message', this.handleFeedData);
    this.socket.removeEventListener('close', this.handleDisconnect);
    this.socket.removeEventListener('error', this.handleDisconnect);
  }

  private handleFeedData = (event: MessageEvent) => {
    let data: FeedMessage.Data;

    if (typeof event.data === 'string') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      data = event.data as any as FeedMessage.Data;
    } else {
      const u8aData = new Uint8Array(event.data);

      // Future-proofing for when we switch to binary feed
      if (u8aData[0] === 0x00) {
        return this.newVersion();
      }

      const str = Connection.utf8decoder.decode(event.data);

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      data = str as any as FeedMessage.Data;
    }

    this.handleMessages(FeedMessage.deserialize(data));
  };

  private autoSubscribe() {
    const { subscribed, chains } = this.appState;
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

    let topChain: Maybe<ChainData> = null;

    for (const chain of chains.values()) {
      if (PINNED_CHAINS[chain.genesisHash] === 1) {
        topChain = chain;
        break;
      }

      if (!topChain || chain.nodeCount > topChain.nodeCount) {
        topChain = chain;
      }
    }

    if (topChain) {
      this.subscribe(topChain.genesisHash);
    }
  }

  private handleDisconnect = async () => {
    console.warn('Disconnecting; will attempt reconnect');
    this.appUpdate({ status: 'offline' });
    this.clean();
    this.socket.close();
    this.socket = await Connection.socket();
    this.bindSocket();
  };
}

/**
 * Fire a function if the timer runs out. You can reset it, or
 * cancel it to prevent the function from being fired.
 *
 * @param onExpired
 * @param timeoutMs
 * @returns
 */
function resettableTimeout(
  onExpired: () => void,
  timeoutMs: Types.Milliseconds
) {
  let timer = setTimeout(onExpired, timeoutMs);

  return {
    reset() {
      clearTimeout(timer);
      timer = setTimeout(onExpired, timeoutMs);
    },
    cancel() {
      clearTimeout(timer);
    },
  };
}
type ResettableTimeout = ReturnType<typeof resettableTimeout>;
