import { Data } from 'ws';
import { Maybe, Types } from '@dotstats/common';

export function parseMessage(data: Data): Maybe<Message> {
  try {
    const message = JSON.parse(data.toString());

    if (message && typeof message.msg === 'string' && typeof message.ts === 'string') {
      message.ts = new Date(message.ts);

      return message;
    }
  } catch (_) {
    console.warn('Error parsing message JSON');
  }

  return null;
}

export function getBestBlock(message: Message): Maybe<BestBlock> {
  switch (message.msg) {
    case 'node.start':
    case 'system.interval':
    case 'block.import':
      return message;
    default:
      return null;
  }
}

interface MessageBase {
  ts: Date,
  level: 'INFO' | 'WARN',
}

export interface BestBlock {
  best: Types.BlockHash;
  height: Types.BlockNumber;
  ts: Date;
}

export interface AfgFinalized {
  ts: Date;
  finalized_number: Types.BlockNumber;
  finalized_hash: Types.BlockHash;
  msg: 'afg.finalized';
}

export interface AfgReceived {
  ts: Date;
  target_number: Maybe<Types.BlockNumber>;
  target_hash: Maybe<Types.BlockHash>;
  voter: Types.Address;
}

export interface AfgReceivedPrecommit extends AfgReceived {
  msg: 'afg.received_precommit';
}

export interface AfgReceivedPrevote extends AfgReceived {
  msg: 'afg.received_prevote';
}

export interface AfgReceivedCommit extends AfgReceived {
  msg: 'afg.received_commit';
}

export interface AfgAuthoritySet {
  msg: 'afg.authority_set';
  ts: Date;
  authority_id: Types.Address,
  authorities: Types.Authorities;
  authority_set_id: Types.AuthoritySetId;
  number: Types.BlockNumber;
  hash: Types.BlockHash;
}

export interface SystemConnected {
  msg: 'system.connected';
  name: Types.NodeName;
  chain: Types.ChainLabel;
  config: string;
  implementation: Types.NodeImplementation;
  version: Types.NodeVersion;
  authority: Maybe<boolean>;
  network_id: Maybe<Types.NetworkId>;
}

export interface SystemInterval extends BestBlock {
  msg: 'system.interval';
  network_state: Maybe<Types.NetworkState>;
  txcount: Types.TransactionCount;
  peers: Types.PeerCount;
  memory: Maybe<Types.MemoryUse>;
  cpu: Maybe<Types.CPUUse>;
  status: 'Idle' | string; // TODO: 'Idle' | ...?
  bandwidth_upload: Maybe<Types.BytesPerSecond>;
  bandwidth_download: Maybe<Types.BytesPerSecond>;
  finalized_height: Maybe<Types.BlockNumber>;
  finalized_hash: Maybe<Types.BlockHash>;
}

export interface SystemNetworkState extends MessageBase {
  msg: 'system.network_state';
  state: Types.NetworkState;
}

export interface NodeStart extends BestBlock {
  msg: 'node.start';
}

export interface BlockImport extends BestBlock {
  msg: 'block.import';
}

// Union type
export type Message = MessageBase & (
  | SystemConnected
  | SystemInterval
  | SystemNetworkState
  | NodeStart
  | BlockImport
  | AfgFinalized
  | AfgReceivedPrecommit
  | AfgReceivedPrevote
  | AfgReceivedCommit
  | AfgAuthoritySet
);


// received: {"msg":"block.import","level":"INFO","ts":"2018-06-18T17:30:35.285406538+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"node.start","level":"INFO","ts":"2018-06-18T17:30:40.038731057+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"system.connected","level":"INFO","ts":"2018-06-18T17:30:40.038975471+02:00","chain":"dev","config":"","version":"0.2.0","implementation":"parity-polkadot","name":"Majestic Widget"}
// received: {"msg":"system.interval","level":"INFO","ts":"2018-06-19T14:00:05.091355364+02:00","txcount":0,"best":"360c9563857308703398f637932b7ffe884e5c7b09692600ff09a4d753c9d948","height":7559,"peers":0,"status":"Idle"}
