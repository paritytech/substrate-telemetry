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
} from './types';

export const ACTIONS = {
  FeedVersion: 0x00 as 0x00,
  BestBlock: 0x01 as 0x01,
  BestFinalized: 0x02 as 0x02,
  AddedNode: 0x03 as 0x03,
  RemovedNode: 0x04 as 0x04,
  LocatedNode: 0x05 as 0x05,
  ImportedBlock: 0x06 as 0x06,
  FinalizedBlock: 0x07 as 0x07,
  NodeStats: 0x08 as 0x08,
  NodeHardware: 0x09 as 0x09,
  TimeSync: 0x0a as 0x0a,
  AddedChain: 0x0b as 0x0b,
  RemovedChain: 0x0c as 0x0c,
  SubscribedTo: 0x0d as 0x0d,
  UnsubscribedFrom: 0x0e as 0x0e,
  Pong: 0x0f as 0x0f,
  AfgFinalized: 0x10 as 0x10,
  AfgReceivedPrevote: 0x11 as 0x11,
  AfgReceivedPrecommit: 0x12 as 0x12,
  AfgAuthoritySet: 0x13 as 0x13,
  StaleNode: 0x14 as 0x14,
  NodeIO: 0x15 as 0x15,
};

export type Action = typeof ACTIONS[keyof typeof ACTIONS];
export type Payload = Message['payload'];

export namespace Variants {
  export interface MessageBase {
    action: Action;
  }

  export interface FeedVersionMessage extends MessageBase {
    action: typeof ACTIONS.FeedVersion;
    payload: FeedVersion;
  }

  export interface BestBlockMessage extends MessageBase {
    action: typeof ACTIONS.BestBlock;
    payload: [BlockNumber, Timestamp, Maybe<Milliseconds>];
  }

  export interface BestFinalizedBlockMessage extends MessageBase {
    action: typeof ACTIONS.BestFinalized;
    payload: [BlockNumber, BlockHash];
  }

  export interface AddedNodeMessage extends MessageBase {
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

  export interface RemovedNodeMessage extends MessageBase {
    action: typeof ACTIONS.RemovedNode;
    payload: NodeId;
  }

  export interface LocatedNodeMessage extends MessageBase {
    action: typeof ACTIONS.LocatedNode;
    payload: [NodeId, Latitude, Longitude, City];
  }

  export interface ImportedBlockMessage extends MessageBase {
    action: typeof ACTIONS.ImportedBlock;
    payload: [NodeId, BlockDetails];
  }

  export interface FinalizedBlockMessage extends MessageBase {
    action: typeof ACTIONS.FinalizedBlock;
    payload: [NodeId, BlockNumber, BlockHash];
  }

  export interface NodeStatsMessage extends MessageBase {
    action: typeof ACTIONS.NodeStats;
    payload: [NodeId, NodeStats];
  }

  export interface NodeHardwareMessage extends MessageBase {
    action: typeof ACTIONS.NodeHardware;
    payload: [NodeId, NodeHardware];
  }

  export interface NodeIOMessage extends MessageBase {
    action: typeof ACTIONS.NodeIO;
    payload: [NodeId, NodeIO];
  }

  export interface TimeSyncMessage extends MessageBase {
    action: typeof ACTIONS.TimeSync;
    payload: Timestamp;
  }

  export interface AddedChainMessage extends MessageBase {
    action: typeof ACTIONS.AddedChain;
    payload: [ChainLabel, GenesisHash, NodeCount];
  }

  export interface RemovedChainMessage extends MessageBase {
    action: typeof ACTIONS.RemovedChain;
    payload: GenesisHash;
  }

  export interface SubscribedToMessage extends MessageBase {
    action: typeof ACTIONS.SubscribedTo;
    payload: GenesisHash;
  }

  export interface UnsubscribedFromMessage extends MessageBase {
    action: typeof ACTIONS.UnsubscribedFrom;
    payload: GenesisHash;
  }

  export interface PongMessage extends MessageBase {
    action: typeof ACTIONS.Pong;
    payload: string; // just echo whatever `ping` sent
  }

  export interface AfgFinalizedMessage extends MessageBase {
    action: typeof ACTIONS.AfgFinalized;
    payload: [Address, BlockNumber, BlockHash];
  }

  export interface AfgAuthoritySet extends MessageBase {
    action: typeof ACTIONS.AfgAuthoritySet;
    payload: AuthoritySetInfo;
  }

  export interface AfgReceivedPrecommit extends MessageBase {
    action: typeof ACTIONS.AfgReceivedPrecommit;
    payload: [Address, BlockNumber, BlockHash, Address];
  }

  export interface AfgReceivedPrevote extends MessageBase {
    action: typeof ACTIONS.AfgReceivedPrevote;
    payload: [Address, BlockNumber, BlockHash, Address];
  }

  export interface StaleNodeMessage extends MessageBase {
    action: typeof ACTIONS.StaleNode;
    payload: NodeId;
  }
}

export type Message =
  | Variants.FeedVersionMessage
  | Variants.BestBlockMessage
  | Variants.BestFinalizedBlockMessage
  | Variants.AddedNodeMessage
  | Variants.RemovedNodeMessage
  | Variants.LocatedNodeMessage
  | Variants.ImportedBlockMessage
  | Variants.FinalizedBlockMessage
  | Variants.NodeStatsMessage
  | Variants.NodeHardwareMessage
  | Variants.TimeSyncMessage
  | Variants.AddedChainMessage
  | Variants.RemovedChainMessage
  | Variants.SubscribedToMessage
  | Variants.UnsubscribedFromMessage
  | Variants.AfgFinalizedMessage
  | Variants.AfgReceivedPrevote
  | Variants.AfgReceivedPrecommit
  | Variants.AfgAuthoritySet
  | Variants.StaleNodeMessage
  | Variants.PongMessage
  | Variants.NodeIOMessage;

/**
 * Data type to be sent to the feed. Passing through strings means we can only serialize once,
 * no matter how many feed clients are listening in.
 */
export interface SquashedMessages extends Array<Action | Payload> {}
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
