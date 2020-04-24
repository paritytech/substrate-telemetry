import * as React from 'react';
import { Types, Maybe, timestamp } from '../../common';
import { State, Node } from '../../state';
import { Truncate } from './';
import { Ago, Icon, Tooltip, Sparkline, PolkadotIcon } from '../';
import {
  formatNumber,
  getHashData,
  milliOrSecond,
  secondsWithPrecision,
} from '../../utils';

export interface Column {
  label: string;
  icon: string;
  width?: number;
  setting?: keyof State.Settings;
  sortBy?: (node: Node) => any;
  render: (node: Node) => React.ReactElement<any> | string;
}

import nodeIcon from '../../icons/server.svg';
import nodeLocationIcon from '../../icons/location.svg';
import nodeValidatorIcon from '../../icons/shield.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import networkIdIcon from '../../icons/fingerprint.svg';
import peersIcon from '../../icons/broadcast.svg';
import transactionsIcon from '../../icons/inbox.svg';
import blockIcon from '../../icons/cube.svg';
import finalizedIcon from '../../icons/cube-alt.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';
import cpuIcon from '../../icons/microchip-solid.svg';
import memoryIcon from '../../icons/memory-solid.svg';
import uploadIcon from '../../icons/cloud-upload.svg';
import downloadIcon from '../../icons/cloud-download.svg';
import readIcon from '../../icons/arrow-up.svg';
import writeIcon from '../../icons/arrow-down.svg';
import databaseIcon from '../../icons/database.svg';
import stateIcon from '../../icons/git-branch.svg';
import networkIcon from '../../icons/network.svg';
import uptimeIcon from '../../icons/pulse.svg';
import externalLinkIcon from '../../icons/link-external.svg';

import parityPolkadotIcon from '../../icons/dot.svg';
import paritySubstrateIcon from '../../icons/substrate.svg';
import polkadotJsIcon from '../../icons/polkadot-js.svg';
import airalabRobonomicsIcon from '../../icons/robonomics.svg';
import chainXIcon from '../../icons/chainx.svg';
import edgewareIcon from '../../icons/edgeware.svg';
import joystreamIcon from '../../icons/joystream.svg';
import ladderIcon from '../../icons/laddernetwork.svg';
import cennznetIcon from '../../icons/cennznet.svg';
import crabIcon from '../../icons/crab.svg';
import darwiniaIcon from '../../icons/darwinia.svg';
import turingIcon from '../../icons/turingnetwork.svg';
import dothereumIcon from '../../icons/dothereum.svg';
import katalchainIcon from '../../icons/katalchain.svg';
import bifrostIcon from '../../icons/bifrost.svg';
import totemIcon from '../../icons/totem.svg';
import nodleIcon from '../../icons/nodle.svg';

import unknownImplementationIcon from '../../icons/question-solid.svg';

const ICONS = {
  'parity-polkadot': parityPolkadotIcon,
  'Parity Polkadot': parityPolkadotIcon,
  'polkadot-js': polkadotJsIcon,
  'airalab-robonomics': airalabRobonomicsIcon,
  'substrate-node': paritySubstrateIcon,
  'Substrate Node': paritySubstrateIcon,
  'edgeware-node': edgewareIcon,
  'Edgeware Node': edgewareIcon,
  'joystream-node': joystreamIcon,
  ChainX: chainXIcon,
  'ladder-node': ladderIcon,
  'cennznet-node': cennznetIcon,
  Crab: crabIcon,
  Darwinia: darwiniaIcon,
  'turing-node': turingIcon,
  dothereum: dothereumIcon,
  katalchain: katalchainIcon,
  'bifrost-node': bifrostIcon,
  'totem-meccano-node': totemIcon,
  Totem: totemIcon,
  'Nodle Chain Node': nodleIcon,
};

export namespace Column {
  export const NAME: Column = {
    label: 'Node',
    icon: nodeIcon,
    sortBy: ({ sortableName }) => sortableName,
    render: ({ name }) => <Truncate text={name} position="left" />,
  };

  export const VALIDATOR: Column = {
    label: 'Validator',
    icon: nodeValidatorIcon,
    width: 16,
    setting: 'validator',
    sortBy: ({ validator }) => validator || '',
    render: ({ validator }) => {
      return validator ? (
        <Tooltip text={validator} copy={true}>
          <span className="Row-validator">
            <PolkadotIcon account={validator} size={16} />
          </span>
        </Tooltip>
      ) : (
        '-'
      );
    },
  };

  export const LOCATION: Column = {
    label: 'Location',
    icon: nodeLocationIcon,
    width: 140,
    setting: 'location',
    sortBy: ({ city }) => city || '',
    render: ({ city }) =>
      city ? <Truncate position="left" text={city} /> : '-',
  };

  export const IMPLEMENTATION: Column = {
    label: 'Implementation',
    icon: nodeTypeIcon,
    width: 90,
    setting: 'implementation',
    sortBy: ({ sortableVersion }) => sortableVersion,
    render: ({ implementation, version }) => {
      const [semver] = version.match(SEMVER_PATTERN) || ['?.?.?'];
      const implIcon = ICONS[implementation] || unknownImplementationIcon;

      return (
        <Tooltip text={`${implementation} v${version}`}>
          <Icon src={implIcon} /> {semver}
        </Tooltip>
      );
    },
  };

  export const NETWORK_ID: Column = {
    label: 'Network ID',
    icon: networkIdIcon,
    width: 90,
    setting: 'networkId',
    sortBy: ({ networkId }) => networkId || '',
    render: ({ networkId }) =>
      networkId ? <Truncate position="left" text={networkId} /> : '-',
  };

  export const PEERS: Column = {
    label: 'Peer Count',
    icon: peersIcon,
    width: 26,
    setting: 'peers',
    sortBy: ({ peers }) => peers,
    render: ({ peers }) => `${peers}`,
  };

  export const TXS: Column = {
    label: 'Transactions in Queue',
    icon: transactionsIcon,
    width: 26,
    setting: 'txs',
    sortBy: ({ txs }) => txs,
    render: ({ txs }) => `${txs}`,
  };

  export const CPU: Column = {
    label: '% CPU Use',
    icon: cpuIcon,
    width: 40,
    setting: 'cpu',
    sortBy: ({ cpu }) => (cpu.length < 3 ? 0 : cpu[cpu.length - 1]),
    render: ({ cpu, chartstamps }) => {
      if (cpu.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatCPU}
          values={cpu}
          stamps={chartstamps}
          minScale={100}
        />
      );
    },
  };

  export const MEM: Column = {
    label: 'Memory Use',
    icon: memoryIcon,
    width: 40,
    setting: 'mem',
    sortBy: ({ mem }) => (mem.length < 3 ? 0 : mem[mem.length - 1]),
    render: ({ mem, chartstamps }) => {
      if (mem.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatMemory}
          values={mem}
          stamps={chartstamps}
          minScale={MEMORY_SCALE}
        />
      );
    },
  };

  export const UPLOAD: Column = {
    label: 'Upload Bandwidth',
    icon: uploadIcon,
    width: 40,
    setting: 'upload',
    sortBy: ({ upload }) => (upload.length < 3 ? 0 : upload[upload.length - 1]),
    render: ({ upload, chartstamps }) => {
      if (upload.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBandwidth}
          values={upload}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      );
    },
  };

  export const DOWNLOAD: Column = {
    label: 'Download Bandwidth',
    icon: downloadIcon,
    width: 40,
    setting: 'download',
    sortBy: ({ download }) =>
      download.length < 3 ? 0 : download[download.length - 1],
    render: ({ download, chartstamps }) => {
      if (download.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBandwidth}
          values={download}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      );
    },
  };

  export const STATE_CACHE: Column = {
    label: 'State Cache Size',
    icon: stateIcon,
    width: 40,
    setting: 'stateCacheSize',
    sortBy: ({ stateCacheSize }) =>
      stateCacheSize.length < 3 ? 0 : stateCacheSize[stateCacheSize.length - 1],
    render: ({ stateCacheSize, chartstamps }) => {
      if (stateCacheSize.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBytes}
          values={stateCacheSize}
          stamps={chartstamps}
          minScale={MEMORY_SCALE}
        />
      );
    },
  };

  export const DB_CACHE: Column = {
    label: 'Database Cache Size',
    icon: databaseIcon,
    width: 40,
    setting: 'dbCacheSize',
    sortBy: ({ dbCacheSize }) =>
      dbCacheSize.length < 3 ? 0 : dbCacheSize[dbCacheSize.length - 1],
    render: ({ dbCacheSize, chartstamps }) => {
      if (dbCacheSize.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBytes}
          values={dbCacheSize}
          stamps={chartstamps}
          minScale={MEMORY_SCALE}
        />
      );
    },
  };

  export const DISK_READ: Column = {
    label: 'Disk Read',
    icon: readIcon,
    width: 40,
    setting: 'diskRead',
    sortBy: ({ diskRead }) =>
      diskRead.length < 3 ? 0 : diskRead[diskRead.length - 1],
    render: ({ diskRead, chartstamps }) => {
      if (diskRead.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBandwidth}
          values={diskRead}
          stamps={chartstamps}
          minScale={MEMORY_SCALE}
        />
      );
    },
  };

  export const DISK_WRITE: Column = {
    label: 'Disk Write',
    icon: writeIcon,
    width: 40,
    setting: 'diskWrite',
    sortBy: ({ diskWrite }) =>
      diskWrite.length < 3 ? 0 : diskWrite[diskWrite.length - 1],
    render: ({ diskWrite, chartstamps }) => {
      if (diskWrite.length < 3) {
        return '-';
      }

      return (
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBandwidth}
          values={diskWrite}
          stamps={chartstamps}
          minScale={MEMORY_SCALE}
        />
      );
    },
  };

  export const BLOCK_NUMBER: Column = {
    label: 'Block',
    icon: blockIcon,
    width: 88,
    setting: 'blocknumber',
    sortBy: ({ height }) => height || 0,
    render: ({ height }) => `#${formatNumber(height)}`,
  };

  export const BLOCK_HASH: Column = {
    label: 'Block Hash',
    icon: blockHashIcon,
    width: 154,
    setting: 'blockhash',
    sortBy: ({ hash }) => hash || '',
    render: ({ hash }) => <Truncate position="right" text={hash} copy={true} />,
  };

  export const FINALIZED: Column = {
    label: 'Finalized Block',
    icon: finalizedIcon,
    width: 88,
    setting: 'finalized',
    sortBy: ({ finalized }) => finalized || 0,
    render: ({ finalized }) => `#${formatNumber(finalized)}`,
  };

  export const FINALIZED_HASH: Column = {
    label: 'Finalized Block Hash',
    icon: blockHashIcon,
    width: 154,
    setting: 'finalizedhash',
    sortBy: ({ finalizedHash }) => finalizedHash || '',
    render: ({ finalizedHash }) => (
      <Truncate position="right" text={finalizedHash} copy={true} />
    ),
  };

  export const BLOCK_TIME: Column = {
    label: 'Block Time',
    icon: blockTimeIcon,
    width: 80,
    setting: 'blocktime',
    sortBy: ({ blockTime }) => (blockTime == null ? Infinity : blockTime),
    render: ({ blockTime }) => `${secondsWithPrecision(blockTime / 1000)}`,
  };

  export const BLOCK_PROPAGATION: Column = {
    label: 'Block Propagation Time',
    icon: propagationTimeIcon,
    width: 58,
    setting: 'blockpropagation',
    sortBy: ({ propagationTime }) =>
      propagationTime == null ? Infinity : propagationTime,
    render: ({ propagationTime }) =>
      propagationTime == null ? '∞' : milliOrSecond(propagationTime),
  };

  export const BLOCK_LAST_TIME: Column = {
    label: 'Last Block Time',
    icon: lastTimeIcon,
    width: 100,
    setting: 'blocklasttime',
    sortBy: ({ blockTimestamp }) => blockTimestamp || 0,
    render: ({ blockTimestamp }) => <Ago when={blockTimestamp} />,
  };

  export const UPTIME: Column = {
    label: 'Node Uptime',
    icon: uptimeIcon,
    width: 58,
    setting: 'uptime',
    sortBy: ({ connectedAt }) => connectedAt || 0,
    render: ({ connectedAt }) => <Ago when={connectedAt} justTime={true} />,
  };

  export const NETWORK_STATE: Column = {
    label: 'NetworkState',
    icon: networkIcon,
    width: 16,
    setting: 'networkstate',
    render: ({ id }) => {
      const chainLabel = getHashData().chain;

      if (!chainLabel) {
        return '-';
      }

      const uri = `${URI_BASE}${encodeURIComponent(chainLabel)}/${id}/`;
      return (
        <a href={uri} target="_blank">
          <Icon src={externalLinkIcon} />
        </a>
      );
    },
  };
}

const SEMVER_PATTERN = /^\d+\.\d+\.\d+/;
const BANDWIDTH_SCALE = 1024 * 1024;
const MEMORY_SCALE = 2 * 1024 * 1024;
const URI_BASE =
  window.location.protocol === 'https:'
    ? `/network_state/`
    : `http://${window.location.hostname}:8000/network_state/`;

function formatStamp(stamp: Types.Timestamp): string {
  const passed = ((timestamp() - stamp) / 1000) | 0;

  const hours = (passed / 3600) | 0;
  const minutes = ((passed % 3600) / 60) | 0;
  const seconds = passed % 60 | 0;

  return hours
    ? `${hours}h ago`
    : minutes
    ? `${minutes}m ago`
    : `${seconds}s ago`;
}

function formatMemory(kbs: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';
  const mbs = (kbs / 1024) | 0;

  if (mbs >= 1000) {
    return `${(mbs / 1024).toFixed(1)} GB${ago}`;
  } else {
    return `${mbs} MB${ago}`;
  }
}

function formatBytes(bytes: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';

  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB${ago}`;
  } else if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB${ago}`;
  } else if (bytes >= 1000) {
    return `${(bytes / 1024).toFixed(1)} kB${ago}`;
  } else {
    return `${bytes} B${ago}`;
  }
}

function formatBandwidth(bps: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';

  if (bps >= 1024 * 1024) {
    return `${(bps / (1024 * 1024)).toFixed(1)} MB/s${ago}`;
  } else if (bps >= 1000) {
    return `${(bps / 1024).toFixed(1)} kB/s${ago}`;
  } else {
    return `${bps | 0} B/s${ago}`;
  }
}

function formatCPU(cpu: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';
  const fractionDigits = cpu > 100 ? 0 : cpu > 10 ? 1 : cpu > 1 ? 2 : 3;

  return `${cpu.toFixed(fractionDigits)}%${ago}`;
}
