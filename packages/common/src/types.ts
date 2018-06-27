import { Opaque } from './helpers';
import { Id } from './id';

export type FeedId = Id<'Feed'>;
export type NodeId = Id<'Node'>;
export type NodeName = Opaque<string, 'NodeName'>;
export type BlockNumber = Opaque<number, 'BlockNumber'>;
export type Milliseconds = Opaque<number, 'Milliseconds'>;

export interface BlockDetails {
    height: BlockNumber;
    blockTime: Milliseconds;
}

export interface NodeDetails {
    name: NodeName;
}

interface BestBlock {
    action: 'best';
    payload: BlockNumber;
}

interface AddedNode {
    action: 'added';
    payload: [NodeId, NodeDetails, BlockDetails];
}

interface RemovedNode {
    action: 'removed';
    payload: NodeId;
}

interface Imported {
    action: 'imported';
    payload: [NodeId, BlockDetails];
}

export type FeedMessage = BestBlock | AddedNode | RemovedNode | Imported;
