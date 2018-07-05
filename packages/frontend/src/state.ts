import { Types } from '@dotstats/common';
import { Node } from './components/Node';

export interface State {
    best: Types.BlockNumber,
    blockTimestamp: Types.Timestamp,
    timeDiff: Types.Milliseconds,
    nodes: Map<Types.NodeId, Node.Props>
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
