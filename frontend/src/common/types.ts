// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import { Opaque, Maybe } from './helpers';
import { Id } from './id';

export type FeedVersion = Opaque<number, 'FeedVersion'>;
export type ChainLabel = Opaque<string, 'ChainLabel'>;
export type GenesisHash = Opaque<string, 'GenesisHash'>;
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

// for verifier
export type AppPeriod = Opaque<number, 'AppPeriod'>;
export type DigestHash = Opaque<string, 'DigestHash'>;
export type VerifierBlockInfos = [
  DigestHash,
  BlockNumber,
  BlockHash,
];

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
  Maybe<NetworkId>,
  Maybe<string>
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
export declare type Precommit = Opaque<boolean, 'Precommit'>;
export declare type Prevote = Opaque<boolean, 'Prevote'>;
export declare type Finalized = Opaque<boolean, 'Finalized'>;
export declare type ImplicitPrecommit = Opaque<boolean, 'ImplicitPrecommit'>;
export declare type ImplicitPrevote = Opaque<boolean, 'ImplicitPrevote'>;
export declare type ImplicitFinalized = Opaque<boolean, 'ImplicitFinalized'>;
export declare type ImplicitPointer = Opaque<BlockNumber, 'ImplicitPointer'>;

export type Ranking<T> = {
  list: Array<[T, number]>;
  other: number;
  unknown: number;
};

export type Range = [number, number | null];

export type ChainStats = {
  version: Maybe<Ranking<string>>;
  target_os: Maybe<Ranking<string>>;
  target_arch: Maybe<Ranking<string>>;
  cpu: Maybe<Ranking<string>>;
  core_count: Maybe<Ranking<number>>;
  memory: Maybe<Ranking<Range>>;
  is_virtual_machine: Maybe<Ranking<boolean>>;
  linux_distro: Maybe<Ranking<string>>;
  linux_kernel: Maybe<Ranking<string>>;
  cpu_hashrate_score: Maybe<Ranking<Range>>;
  memory_memcpy_score: Maybe<Ranking<Range>>;
  disk_sequential_write_score: Maybe<Ranking<Range>>;
  disk_random_write_score: Maybe<Ranking<Range>>;
};
