import { Types, Maybe } from '@dotstats/common';

export namespace State {
  export interface Node {
    pinned: boolean,
    id: Types.NodeId;
    nodeDetails: Types.NodeDetails;
    nodeStats: Types.NodeStats;
    blockDetails: Types.BlockDetails;
    location: Maybe<Types.NodeLocation>;
  }

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
  nodes: Map<Types.NodeId, State.Node>;
  settings: Readonly<State.Settings>;
  pins: Readonly<Set<Types.NodeName>>;
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
