import { Opaque, Maybe } from './helpers';
import { Id } from './id';

export type FeedVersion = Opaque<number, 'FeedVersion'>;
export type ChainLabel = Opaque<string, 'ChainLabel'>;
export type FeedId = Id<'Feed'>;
export type NodeId = Id<'Node'>;
export type NodeName = Opaque<string, 'NodeName'>;
export type NodeImplementation = Opaque<string, 'NodeImplementation'>;
export type NodeVersion = Opaque<string, 'NodeVersion'>;
export type BlockNumber = Opaque<number, 'BlockNumber'>;
export type BlockHash = Opaque<string, 'BlockHash'>;
export type Address = Opaque<string, 'Address'>;
export type Milliseconds = Opaque<number, 'Milliseconds'>;
export type Timestamp = Opaque<Milliseconds, 'Timestamp'>;
export type PropagationTime = Opaque<Milliseconds, 'PropagationTime'>;
export type NodeCount = Opaque<number, 'NodeCount'>;
export type PeerCount = Opaque<number, 'PeerCount'>;
export type TransactionCount = Opaque<number, 'TransactionCount'>;
export type Latitude = Opaque<number, 'Latitude'>;
export type Longitude = Opaque<number, 'Longitude'>;
export type City = Opaque<string, 'City'>;
export type MemoryUse = Opaque<number, 'MemoryUse'>;
export type CPUUse = Opaque<number, 'CPUUse'>;
export type Bytes = Opaque<number, 'Bytes'>;
export type BytesPerSecond = Opaque<number, 'BytesPerSecond'>;
export type NetworkId = Opaque<string, 'NetworkId'>;
export type NetworkState = Opaque<string | object, 'NetworkState'>;

export type BlockDetails = [
  BlockNumber,
  BlockHash,
  Milliseconds,
  Timestamp,
  Maybe<PropagationTime>
];
export type NodeDetails = [
  NodeName,
  NodeImplementation,
  NodeVersion,
  Maybe<Address>,
  Maybe<NetworkId>
];
export type NodeStats = [PeerCount, TransactionCount];
export type NodeIO = [Array<Bytes>];
export type NodeHardware = [
  Array<BytesPerSecond>,
  Array<BytesPerSecond>,
  Array<Timestamp>
];
export type NodeLocation = [Latitude, Longitude, City];

export interface Authority {
  Address: Address;
  NodeId: Maybe<NodeId>;
  Name: Maybe<NodeName>;
}
export declare type Authorities = Array<Address>;
export declare type AuthoritySetId = Opaque<number, 'AuthoritySetId'>;
export declare type AuthoritySetInfo = [
  AuthoritySetId,
  Authorities,
  Address,
  BlockNumber,
  BlockHash
];
export declare type ConsensusItem = [BlockNumber, ConsensusView];
export declare type ConsensusInfo = Array<ConsensusItem>;
export declare type ConsensusView = Map<Address, ConsensusState>;
export declare type ConsensusState = Map<Address, ConsensusDetail>;
export interface ConsensusDetail {
  Precommit: Precommit;
  ImplicitPrecommit: ImplicitPrecommit;
  Prevote: Prevote;
  ImplicitPrevote: ImplicitPrevote;
  ImplicitPointer: ImplicitPointer;
  Finalized: ImplicitFinalized;
  ImplicitFinalized: Finalized;
  FinalizedHash: BlockHash;
  FinalizedHeight: BlockNumber;
}
export declare type Precommit = Opaque<boolean, 'Precommit'>;
export declare type Prevote = Opaque<boolean, 'Prevote'>;
export declare type Finalized = Opaque<boolean, 'Finalized'>;
export declare type ImplicitPrecommit = Opaque<boolean, 'ImplicitPrecommit'>;
export declare type ImplicitPrevote = Opaque<boolean, 'ImplicitPrevote'>;
export declare type ImplicitFinalized = Opaque<boolean, 'ImplicitFinalized'>;
export declare type ImplicitPointer = Opaque<BlockNumber, 'ImplicitPointer'>;
