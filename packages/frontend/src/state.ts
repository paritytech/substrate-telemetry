import { Types } from '@dotstats/common';
import { Node } from './Node';

export interface State {
    best: Types.BlockNumber,
    nodes: Map<Types.NodeId, Node.Props>
}

export type Update = <K extends keyof State>(changes: Pick<State, K> | null) => Readonly<State>;
