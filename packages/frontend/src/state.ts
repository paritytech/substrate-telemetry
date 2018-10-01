import { Types, Maybe } from '@dotstats/common';

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
  public readonly name: Types.NodeName;
  public readonly implementation: Types.NodeImplementation;
  public readonly version: Types.NodeVersion;
  public readonly validator: Maybe<Types.Address>;

  public pinned: boolean;
  public peers: Types.PeerCount;
  public txs: Types.TransactionCount;
  public mem: Types.MemoryUse[];
  public cpu: Types.CPUUse[];
  public chartstamps: Types.Timestamp[];

  public height: Types.BlockNumber;
  public hash: Types.BlockHash;
  public blockTime: Types.Milliseconds;
  public blockTimestamp: Types.Timestamp;
  public propagationTime: Maybe<Types.PropagationTime>;

  public lat: Maybe<Types.Latitude>;
  public lon: Maybe<Types.Longitude>;
  public city: Maybe<Types.City>;

  constructor(
    pinned: boolean,
    id: Types.NodeId,
    nodeDetails: Types.NodeDetails,
    nodeStats: Types.NodeStats,
    blockDetails: Types.BlockDetails,
    location: Maybe<Types.NodeLocation>
  ) {
    const [name, implementation, version, validator] = nodeDetails;

    this.pinned = pinned;

    this.id = id;
    this.name = name;
    this.implementation = implementation;
    this.version = version;
    this.validator = validator;

    this.updateStats(nodeStats);
    this.updateBlock(blockDetails);

    if (location) {
      this.updateLocation(location);
    }
  }

  public updateStats(stats: Types.NodeStats) {
    const [peers, txs, mem, cpu, chartstamps] = stats;

    this.peers = peers;
    this.txs = txs;
    this.mem = mem;
    this.cpu = cpu;
    this.chartstamps = chartstamps;
  }

  public updateBlock(block: Types.BlockDetails) {
    const [height, hash, blockTime, blockTimestamp, propagationTime] = block;

    this.height = height;
    this.hash = hash;
    this.blockTime = blockTime;
    this.blockTimestamp = blockTimestamp;
    this.propagationTime = propagationTime;
  }

  public updateLocation(location: Types.NodeLocation) {
    const [lat, lon, city] = location;

    this.lat = lat;
    this.lon = lon;
    this.city = city;
  }
}

export namespace State {
  export interface Settings {
    location: boolean;
    validator: boolean;
    implementation: boolean;
    peers: boolean;
    txs: boolean;
    cpu: boolean;
    mem: boolean;
    blocknumber: boolean;
    blockhash: boolean;
    blocktime: boolean;
    blockpropagation: boolean;
    blocklasttime: boolean;
  }
}

export interface State {
  status: 'online' | 'offline' | 'upgrade-requested';
  best: Types.BlockNumber;
  blockTimestamp: Types.Timestamp;
  blockAverage: Maybe<Types.Milliseconds>;
  timeDiff: Types.Milliseconds;
  subscribed: Maybe<Types.ChainLabel>;
  chains: Map<Types.ChainLabel, Types.NodeCount>;
  nodes: Map<Types.NodeId, Node>;
  sortedNodes: Node[];
  settings: Readonly<State.Settings>;
  pins: Readonly<Set<Types.NodeName>>;
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;

