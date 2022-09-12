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

import * as React from 'react';
import { Types, Maybe, SortedCollection } from './common';
import { Column } from './components/List';

export const PINNED_CHAINS = {
  // Kusama
  '0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe': 2,
  // Polkadot
  '0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3': 1,
};

export function comparePinnedChains(a: string, b: string) {
  const aWeight = PINNED_CHAINS[a] || 1024;
  const bWeight = PINNED_CHAINS[b] || 1024;

  return aWeight - bWeight;
}

export class Node {
  public static compare(a: Node, b: Node): number {
    if (a.pinned === b.pinned && a.stale === b.stale) {
      if (a.height === b.height) {
        const aPropagation =
          a.propagationTime == null ? Infinity : (a.propagationTime as number);
        const bPropagation =
          b.propagationTime == null ? Infinity : (b.propagationTime as number);

        // Ascending sort by propagation time
        return aPropagation - bPropagation;
      }
    } else {
      const bSort = (b.pinned ? -2 : 0) + +b.stale;
      const aSort = (a.pinned ? -2 : 0) + +a.stale;

      return aSort - bSort;
    }

    // Descending sort by block number
    return b.height - a.height;
  }

  public readonly id: Types.NodeId;
  public readonly name: Types.NodeName;
  public readonly implementation: Types.NodeImplementation;
  public readonly version: Types.NodeVersion;
  public readonly validator: Maybe<Types.Address>;
  public readonly networkId: Maybe<Types.NetworkId>;
  public readonly startupTime: Maybe<Types.Timestamp>;

  public readonly sortableName: string;
  public readonly sortableVersion: number;

  public stale: boolean;
  public pinned: boolean;
  public peers: Types.PeerCount;
  public txs: Types.TransactionCount;
  public upload: Types.BytesPerSecond[];
  public download: Types.BytesPerSecond[];
  public stateCacheSize: Types.Bytes[];
  public chartstamps: Types.Timestamp[];

  public height: Types.BlockNumber;
  public hash: Types.BlockHash;
  public blockTime: Types.Milliseconds;
  public blockTimestamp: Types.Timestamp;
  public propagationTime: Maybe<Types.PropagationTime>;

  public finalized = 0 as Types.BlockNumber;
  public finalizedHash = '' as Types.BlockHash;

  public lat: Maybe<Types.Latitude>;
  public lon: Maybe<Types.Longitude>;
  public city: Maybe<Types.City>;

  private _changeRef = 0;
  private readonly subscriptionsConsensus = new Set<(node: Node) => void>();

  constructor(
    pinned: boolean,
    id: Types.NodeId,
    nodeDetails: Types.NodeDetails,
    nodeStats: Types.NodeStats,
    nodeIO: Types.NodeIO,
    nodeHardware: Types.NodeHardware,
    blockDetails: Types.BlockDetails,
    location: Maybe<Types.NodeLocation>,
    startupTime: Maybe<Types.Timestamp>
  ) {
    const [name, implementation, version, validator, networkId] = nodeDetails;

    this.pinned = pinned;

    this.id = id;
    this.name = name;
    this.implementation = implementation;
    this.version = version;
    this.validator = validator;
    this.networkId = networkId;
    this.startupTime = startupTime;

    const [major = 0, minor = 0, patch = 0] = (version || '0.0.0')
      .split('.')
      .map((n) => parseInt(n, 10) | 0);

    this.sortableName = name.toLocaleLowerCase();
    this.sortableVersion = (major * 1000 + minor * 100 + patch) | 0;

    this.updateStats(nodeStats);
    this.updateIO(nodeIO);
    this.updateHardware(nodeHardware);
    this.updateBlock(blockDetails);

    if (location) {
      this.updateLocation(location);
    }
  }

  public updateStats(stats: Types.NodeStats) {
    const [peers, txs] = stats;

    this.peers = peers;
    this.txs = txs;

    this.trigger();
  }

  public updateIO(io: Types.NodeIO) {
    const [stateCacheSize] = io;

    this.stateCacheSize = stateCacheSize;

    this.trigger();
  }

  public updateHardware(hardware: Types.NodeHardware) {
    const [upload, download, chartstamps] = hardware;

    this.upload = upload;
    this.download = download;
    this.chartstamps = chartstamps;

    this.trigger();
  }

  public updateBlock(block: Types.BlockDetails) {
    const [height, hash, blockTime, blockTimestamp, propagationTime] = block;

    this.height = height;
    this.hash = hash;
    this.blockTime = blockTime;
    this.blockTimestamp = blockTimestamp;
    this.propagationTime = propagationTime;
    this.stale = false;

    this.trigger();
  }

  public updateFinalized(height: Types.BlockNumber, hash: Types.BlockHash) {
    this.finalized = height;
    this.finalizedHash = hash;
  }

  public updateLocation(location: Types.NodeLocation) {
    const [lat, lon, city] = location;

    this.lat = lat;
    this.lon = lon;
    this.city = city;

    this.trigger();
  }

  public newBestBlock() {
    if (this.propagationTime != null) {
      this.propagationTime = null;
      this.trigger();
    }
  }

  public setPinned(pinned: boolean) {
    if (this.pinned !== pinned) {
      this.pinned = pinned;
      this.trigger();
    }
  }

  public setStale(stale: boolean) {
    if (this.stale !== stale) {
      this.stale = stale;
      this.trigger();
    }
  }

  public get changeRef(): number {
    return this._changeRef;
  }

  private trigger() {
    this._changeRef += 1;
  }
}

export function bindState(bind: React.Component, state: State): Update {
  let isUpdating = false;

  return (changes) => {
    // Apply new changes to the state immediately
    Object.assign(state, changes);

    // Trigger React update on next animation frame only once
    if (!isUpdating) {
      isUpdating = true;

      window.requestAnimationFrame(() => {
        bind.forceUpdate();
        isUpdating = false;
      });
    }

    return state;
  };
}

export interface StateSettings {
  location: boolean;
  validator: boolean;
  implementation: boolean;
  networkId: boolean;
  peers: boolean;
  txs: boolean;
  upload: boolean;
  download: boolean;
  stateCacheSize: boolean;
  blocknumber: boolean;
  blockhash: boolean;
  finalized: boolean;
  finalizedhash: boolean;
  blocktime: boolean;
  blockpropagation: boolean;
  blocklasttime: boolean;
  uptime: boolean;
}

export interface State {
  status: 'online' | 'offline' | 'upgrade-requested';
  best: Types.BlockNumber;
  finalized: Types.BlockNumber;
  tab: string;
  blockTimestamp: Types.Timestamp;
  blockAverage: Maybe<Types.Milliseconds>;
  timeDiff: Types.Milliseconds;
  subscribed: Maybe<Types.GenesisHash>;
  chains: Map<Types.GenesisHash, ChainData>;
  nodes: SortedCollection<Node>;
  settings: Readonly<StateSettings>;
  pins: Readonly<Set<Types.NodeName>>;
  sortBy: Readonly<Maybe<number>>;
  selectedColumns: Column[];
  chainStats: Maybe<Types.ChainStats>;
}

export type Update = <K extends keyof State>(
  changes: Pick<State, K>
) => Readonly<State>;

export interface ChainData {
  label: Types.ChainLabel;
  genesisHash: Types.GenesisHash;
  nodeCount: Types.NodeCount;
}
