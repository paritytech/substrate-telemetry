import { Opaque } from './helpers';
import { Id } from './id';

export type FeedId = Id<'Feed'>;
export type NodeId = Id<'Node'>;
export type NodeName = Opaque<string, 'NodeName'>;
export type NodeImplementation = Opaque<string, 'NodeImplementation'>;
export type NodeVersion = Opaque<string, 'NodeVersion'>;
export type BlockNumber = Opaque<number, 'BlockNumber'>;
export type BlockHash = Opaque<string, 'BlockHash'>;
export type Milliseconds = Opaque<number, 'Milliseconds'>;
export type PeerCount = Opaque<number, 'PeerCount'>;
export type TransactionCount = Opaque<number, 'TransactionCount'>;

export type BlockDetails = [BlockNumber, BlockHash, Milliseconds];
export type NodeDetails = [NodeName, NodeImplementation, NodeVersion];
export type NodeStats = [PeerCount, TransactionCount];

interface BestBlock {
    action: 'best';
    payload: BlockNumber;
}

interface AddedNode {
    action: 'added';
    payload: [NodeId, NodeDetails, NodeStats, BlockDetails];
}

interface RemovedNode {
    action: 'removed';
    payload: NodeId;
}

interface Imported {
    action: 'imported';
    payload: [NodeId, BlockDetails];
}

interface Stats {
    action: 'stats';
    payload: [NodeId, NodeStats];
}

export type FeedMessage = BestBlock | AddedNode | RemovedNode | Imported | Stats;
