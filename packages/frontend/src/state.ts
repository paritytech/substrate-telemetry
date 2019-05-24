import { Types, Maybe, SortedCollection } from '@dotstats/common';

export class Node {
  public static compare(a: Node, b: Node): number {
    if (a.pinned === b.pinned) {
      if (a.height === b.height) {
        const aPropagation = a.propagationTime == null ? Infinity : a.propagationTime as number;
        const bPropagation = b.propagationTime == null ? Infinity : b.propagationTime as number;

        // Ascending sort by propagation time
        return aPropagation - bPropagation;
      }
    } else {
      return +b.pinned - +a.pinned;
    }

    // Descending sort by block number
    return b.height - a.height;
  }

  public readonly id: Types.NodeId;
  public readonly address: Types.Address;
  public readonly name: Types.NodeName;
  public readonly implementation: Types.NodeImplementation;
  public readonly version: Types.NodeVersion;
  public readonly validator: Maybe<Types.Address>;
  public readonly networkId: Maybe<Types.NetworkId>;

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
    location: Maybe<Types.NodeLocation>
  ) {
    const [name, implementation, version, validator, networkId, address] = nodeDetails;

    this.pinned = pinned;

    this.id = id;
    this.name = name;
    this.address = address;
    this.implementation = implementation;
    this.version = version;
    this.validator = validator;
    this.networkId = networkId;

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
    networkstate: boolean;
  }
}

export interface State {
  status: 'online' | 'offline' | 'upgrade-requested';
  best: Types.BlockNumber;
  finalized: Types.BlockNumber;
  consensusInfo: Types.ConsensusInfo;
  displayConsensusLoadingScreen: boolean;
  tabChanged: boolean;
  authorities: Types.Address[];
  authoritySetId: Maybe<Types.AuthoritySetId>;
  sendFinality: boolean;
  blockTimestamp: Types.Timestamp;
  blockAverage: Maybe<Types.Milliseconds>;
  timeDiff: Types.Milliseconds;
  subscribed: Maybe<Types.ChainLabel>;
  chains: Map<Types.ChainLabel, Types.NodeCount>;
  nodes: SortedCollection<Types.NodeId, Node>;
  settings: Readonly<State.Settings>;
  pins: Readonly<Set<Types.NodeName>>;
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
export type UpdateBound = <K extends keyof State>(changes: Pick<State, K> | null) => void;
