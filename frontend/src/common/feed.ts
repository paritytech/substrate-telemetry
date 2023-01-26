// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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

import { Maybe } from './helpers';
import { stringify, parse, Stringified } from './stringify';
import {
  FeedVersion,
  Address,
  Latitude,
  Longitude,
  City,
  NodeId,
  NodeCount,
  NodeDetails,
  NodeStats,
  NodeIO,
  NodeHardware,
  NodeLocation,
  BlockNumber,
  BlockHash,
  BlockDetails,
  Timestamp,
  Milliseconds,
  ChainLabel,
  GenesisHash,
  AuthoritySetInfo,
  ChainStats,
} from './types';

export const ACTIONS = {
  FeedVersion: 0x00 as const,
  BestBlock: 0x01 as const,
  BestFinalized: 0x02 as const,
  AddedNode: 0x03 as const,
  RemovedNode: 0x04 as const,
  LocatedNode: 0x05 as const,
  ImportedBlock: 0x06 as const,
  FinalizedBlock: 0x07 as const,
  NodeStats: 0x08 as const,
  NodeHardware: 0x09 as const,
  TimeSync: 0x0a as const,
  AddedChain: 0x0b as const,
  RemovedChain: 0x0c as const,
  SubscribedTo: 0x0d as const,
  UnsubscribedFrom: 0x0e as const,
  Pong: 0x0f as const,
  AfgFinalized: 0x10 as const,
  AfgReceivedPrevote: 0x11 as const,
  AfgReceivedPrecommit: 0x12 as const,
  AfgAuthoritySet: 0x13 as const,
  StaleNode: 0x14 as const,
  NodeIO: 0x15 as const,
  ChainStatsUpdate: 0x16 as const,
};

export type Action = typeof ACTIONS[keyof typeof ACTIONS];
export type Payload = Message['payload'];

interface MessageBase {
  action: Action;
}

interface FeedVersionMessage extends MessageBase {
  action: typeof ACTIONS.FeedVersion;
  payload: FeedVersion;
}

interface BestBlockMessage extends MessageBase {
  action: typeof ACTIONS.BestBlock;
  payload: [BlockNumber, Timestamp, Maybe<Milliseconds>];
}

interface BestFinalizedBlockMessage extends MessageBase {
  action: typeof ACTIONS.BestFinalized;
  payload: [BlockNumber, BlockHash];
}

interface AddedNodeMessage extends MessageBase {
  action: typeof ACTIONS.AddedNode;
  payload: [
    NodeId,
    NodeDetails,
    NodeStats,
    NodeIO,
    NodeHardware,
    BlockDetails,
    Maybe<NodeLocation>,
    Maybe<Timestamp>
  ];
}

interface RemovedNodeMessage extends MessageBase {
  action: typeof ACTIONS.RemovedNode;
  payload: NodeId;
}

interface LocatedNodeMessage extends MessageBase {
  action: typeof ACTIONS.LocatedNode;
  payload: [NodeId, Latitude, Longitude, City];
}

interface ImportedBlockMessage extends MessageBase {
  action: typeof ACTIONS.ImportedBlock;
  payload: [NodeId, BlockDetails];
}

interface FinalizedBlockMessage extends MessageBase {
  action: typeof ACTIONS.FinalizedBlock;
  payload: [NodeId, BlockNumber, BlockHash];
}

interface NodeStatsMessage extends MessageBase {
  action: typeof ACTIONS.NodeStats;
  payload: [NodeId, NodeStats];
}

interface NodeHardwareMessage extends MessageBase {
  action: typeof ACTIONS.NodeHardware;
  payload: [NodeId, NodeHardware];
}

interface NodeIOMessage extends MessageBase {
  action: typeof ACTIONS.NodeIO;
  payload: [NodeId, NodeIO];
}

interface TimeSyncMessage extends MessageBase {
  action: typeof ACTIONS.TimeSync;
  payload: Timestamp;
}

interface AddedChainMessage extends MessageBase {
  action: typeof ACTIONS.AddedChain;
  payload: [ChainLabel, GenesisHash, NodeCount];
}

interface RemovedChainMessage extends MessageBase {
  action: typeof ACTIONS.RemovedChain;
  payload: GenesisHash;
}

interface SubscribedToMessage extends MessageBase {
  action: typeof ACTIONS.SubscribedTo;
  payload: GenesisHash;
}

interface UnsubscribedFromMessage extends MessageBase {
  action: typeof ACTIONS.UnsubscribedFrom;
  payload: GenesisHash;
}

interface PongMessage extends MessageBase {
  action: typeof ACTIONS.Pong;
  payload: string; // just echo whatever `ping` sent
}

interface AfgFinalizedMessage extends MessageBase {
  action: typeof ACTIONS.AfgFinalized;
  payload: [Address, BlockNumber, BlockHash];
}

interface AfgAuthoritySet extends MessageBase {
  action: typeof ACTIONS.AfgAuthoritySet;
  payload: AuthoritySetInfo;
}

interface AfgReceivedPrecommit extends MessageBase {
  action: typeof ACTIONS.AfgReceivedPrecommit;
  payload: [Address, BlockNumber, BlockHash, Address];
}

interface AfgReceivedPrevote extends MessageBase {
  action: typeof ACTIONS.AfgReceivedPrevote;
  payload: [Address, BlockNumber, BlockHash, Address];
}

interface StaleNodeMessage extends MessageBase {
  action: typeof ACTIONS.StaleNode;
  payload: NodeId;
}

interface ChainStatsUpdate extends MessageBase {
  action: typeof ACTIONS.ChainStatsUpdate;
  payload: ChainStats;
}

export type Message =
  | FeedVersionMessage
  | BestBlockMessage
  | BestFinalizedBlockMessage
  | AddedNodeMessage
  | RemovedNodeMessage
  | LocatedNodeMessage
  | ImportedBlockMessage
  | FinalizedBlockMessage
  | NodeStatsMessage
  | NodeHardwareMessage
  | TimeSyncMessage
  | AddedChainMessage
  | RemovedChainMessage
  | SubscribedToMessage
  | UnsubscribedFromMessage
  | AfgFinalizedMessage
  | AfgReceivedPrevote
  | AfgReceivedPrecommit
  | AfgAuthoritySet
  | StaleNodeMessage
  | PongMessage
  | NodeIOMessage
  | ChainStatsUpdate;

/**
 * Data type to be sent to the feed. Passing through strings means we can only serialize once,
 * no matter how many feed clients are listening in.
 */
export type SquashedMessages = Array<Action | Payload>;
export type Data = Stringified<SquashedMessages>;

/**
 * Serialize an array of `Message`s to a single JSON string.
 *
 * All messages are squashed into a single array of alternating opcodes and payloads.
 *
 * Action `string`s are converted to opcodes using the `actionToCode` mapping.
 */
export function serialize(messages: Array<Message>): Data {
  const squashed: SquashedMessages = new Array(messages.length * 2);
  let index = 0;

  messages.forEach((message) => {
    const { action, payload } = message;

    squashed[index++] = action;
    squashed[index++] = payload;
  });

  return stringify(squashed);
}

/**
 * Deserialize data to an array of `Message`s.
 */
export function deserialize(data: Data): Array<Message> {
  const json = parse(data);

  if (!Array.isArray(json) || json.length === 0 || json.length % 2 !== 0) {
    throw new Error('Invalid FeedMessage.Data');
  }

  const messages = new Array<Message>(json.length / 2);

  for (const index of messages.keys()) {
    const [action, payload] = json.slice(index * 2);

    messages[index] = { action, payload } as Message;
  }

  return messages;
}
