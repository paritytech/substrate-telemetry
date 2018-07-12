import { Node } from './components/Node';
import { Types, Maybe } from '@dotstats/common';

export interface State {
  best: Types.BlockNumber,
  blockTimestamp: Types.Timestamp,
  timeDiff: Types.Milliseconds,
  subscribed: Maybe<Types.ChainLabel>,
  chains: Map<Types.ChainLabel, Types.NodeCount>,
  nodes: Map<Types.NodeId, Node.Props>,
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
