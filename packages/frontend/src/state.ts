import { Types, Maybe, SortedCollection } from '@dotstats/common';
import { Column } from './components/List';

export const PINNED_CHAIN = 'Kusama CC3';

export class Node {
  public static compare(a: Node, b: Node): number {
    if (a.pinned === b.pinned && a.stale === b.stale) {
      if (a.height === b.height) {
        const aPropagation = a.propagationTime == null ? Infinity : a.propagationTime as number;
        const bPropagation = b.propagationTime == null ? Infinity : b.propagationTime as number;

        // Ascending sort by propagation time
        return aPropagation - bPropagation;
      }
    } else {
      const bSort = (b.pinned ? -2 : 0) + +b.stale;
      const aSort = (a.pinned ? -2 : 0) + +a.stale;

      return aSort - bSort;
    }

    // Descending sort by block number
    return b.height - a.height;
  }

  public readonly id: Types.NodeId;
  public readonly name: Types.NodeName;
  public readonly implementation: Types.NodeImplementation;
  public readonly version: Types.NodeVersion;
  public readonly validator: Maybe<Types.Address>;
  public readonly networkId: Maybe<Types.NetworkId>;
  public readonly connectedAt: Types.Timestamp;

  public readonly sortableName: string;
  public readonly sortableVersion: number;

  public stale: boolean;
  public pinned: boolean;
  public peers: Types.PeerCount;
  public txs: Types.TransactionCount;
  public mem: Types.MemoryUse[];
  public cpu: Types.CPUUse[];
  public upload: Types.BytesPerSecond[];
  public download: Types.BytesPerSecond[];
  public chartstamps: Types.Timestamp[];

  public height: Types.BlockNumber;
  public hash: Types.BlockHash;
  public blockTime: Types.Milliseconds;
  public blockTimestamp: Types.Timestamp;
  public propagationTime: Maybe<Types.PropagationTime>;

  public finalized = 0 as Types.BlockNumber;
  public finalizedHash = '' as Types.BlockHash;

  public lat: Maybe<Types.Latitude>;
  public lon: Maybe<Types.Longitude>;
  public city: Maybe<Types.City>;

  private readonly subscriptions = new Set<(node: Node) => void>();
  private readonly subscriptionsConsensus = new Set<(node: Node) => void>();

  constructor(
    pinned: boolean,
    id: Types.NodeId,
    nodeDetails: Types.NodeDetails,
    nodeStats: Types.NodeStats,
    nodeHardware: Types.NodeHardware,
    blockDetails: Types.BlockDetails,
    location: Maybe<Types.NodeLocation>,
    connectedAt: Types.Timestamp,
  ) {
    const [name, implementation, version, validator, networkId] = nodeDetails;

    this.pinned = pinned;

    this.id = id;
    this.name = name;
    this.implementation = implementation;
    this.version = version;
    this.validator = validator;
    this.networkId = networkId;
    this.connectedAt = connectedAt;

    const [major = 0, minor = 0, patch = 0] = (version || '0.0.0').split('.').map((n) => parseInt(n, 10) | 0);

    this.sortableName = name.toLocaleLowerCase();
    this.sortableVersion = (major * 1000 + minor * 100 + patch) | 0;

    this.updateStats(nodeStats);
    this.updateHardware(nodeHardware);
    this.updateBlock(blockDetails);

    if (location) {
      this.updateLocation(location);
    }
  }

  public updateStats(stats: Types.NodeStats) {
    const [peers, txs] = stats;

    this.peers = peers;
    this.txs = txs;

    this.trigger();
  }

  public updateHardware(hardware: Types.NodeHardware) {
    const [mem, cpu, upload, download, chartstamps] = hardware;

    this.mem = mem;
    this.cpu = cpu;
    this.upload = upload;
    this.download = download;
    this.chartstamps = chartstamps;

    this.trigger();
  }

  public updateBlock(block: Types.BlockDetails) {
    const [height, hash, blockTime, blockTimestamp, propagationTime] = block;

    this.height = height;
    this.hash = hash;
    this.blockTime = blockTime;
    this.blockTimestamp = blockTimestamp;
    this.propagationTime = propagationTime;
    this.stale = false;

    this.trigger();
  }

  public updateFinalized(height: Types.BlockNumber, hash: Types.BlockHash) {
    this.finalized = height;
    this.finalizedHash = hash;
  }

  public updateLocation(location: Types.NodeLocation) {
    const [lat, lon, city] = location;

    this.lat = lat;
    this.lon = lon;
    this.city = city;

    this.trigger();
  }

  public newBestBlock() {
    if (this.propagationTime != null) {
      this.propagationTime = null;
      this.trigger();
    }
  }

  public setPinned(pinned: boolean) {
    if (this.pinned !== pinned) {
      this.pinned = pinned;
      this.trigger();
    }
  }

  public setStale(stale: boolean) {
    if (this.stale !== stale) {
      this.stale = stale;
      this.trigger();
    }
  }

  public subscribe(handler: (node: Node) => void) {
    this.subscriptions.add(handler);
  }

  public unsubscribe(handler: (node: Node) => void) {
    this.subscriptions.delete(handler);
  }

  public subscribeConsensus(handler: (node: Node) => void) {
    this.subscriptionsConsensus.add(handler);
  }

  public unsubscribeConsensus(handler: (node: Node) => void) {
    this.subscriptionsConsensus.delete(handler);
  }

  private trigger() {
    for (const handler of this.subscriptions.values()) {
      handler(this);
    }
  }
}

export namespace State {
  export interface Settings {
    location: boolean;
    validator: boolean;
    implementation: boolean;
    networkId: boolean;
    peers: boolean;
    txs: boolean;
    cpu: boolean;
    mem: boolean;
    upload: boolean;
    download: boolean;
    blocknumber: boolean;
    blockhash: boolean;
    finalized: boolean;
    finalizedhash: boolean;
    blocktime: boolean;
    blockpropagation: boolean;
    blocklasttime: boolean;
    uptime: boolean;
    networkstate: boolean;
  }

  export interface SortBy {
    column: string;
    reverse: boolean;
  }
}

export interface State {
  status: 'online' | 'offline' | 'upgrade-requested';
  best: Types.BlockNumber;
  finalized: Types.BlockNumber;
  consensusInfo: Types.ConsensusInfo;
  displayConsensusLoadingScreen: boolean;
  tab: string;
  authorities: Types.Address[];
  authoritySetId: Maybe<Types.AuthoritySetId>;
  sendFinality: boolean;
  blockTimestamp: Types.Timestamp;
  blockAverage: Maybe<Types.Milliseconds>;
  timeDiff: Types.Milliseconds;
  subscribed: Maybe<Types.ChainLabel>;
  chains: Map<Types.ChainLabel, ChainData>;
  nodes: SortedCollection<Node>;
  settings: Readonly<State.Settings>;
  pins: Readonly<Set<Types.NodeName>>;
  sortBy: Readonly<Maybe<number>>;
  selectedColumns: Column[];
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
export type UpdateBound = <K extends keyof State>(changes: Pick<State, K> | null) => void;

export interface ChainData {
  label: Types.ChainLabel;
  nodeCount: Types.NodeCount;
}
