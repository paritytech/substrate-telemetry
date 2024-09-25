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

import { Types, Maybe, timestamp } from '../../../common';
import { Node } from '../../../state';

import './Column.css';

import {
  NameColumn,
  ValidatorColumn,
  LocationColumn,
  ImplementationColumn,
  NetworkIdColumn,
  PeersColumn,
  TxsColumn,
  UploadColumn,
  DownloadColumn,
  StateCacheColumn,
  BlockNumberColumn,
  BlockHashColumn,
  FinalizedBlockColumn,
  FinalizedHashColumn,
  BlockTimeColumn,
  BlockPropagationColumn,
  LastBlockColumn,
  UptimeColumn,
  CpuArchitectureColumn, //extra columns added
  CpuColumn,
  CpuCoresColumn,
  LinuxKernelColumn,
  IsVirtualMachineColumn,
  MemoryColumn,
  OperatingSystemColumn,
  VersionColumn,
  LinuxDistroColumn,
} from './';

export type Column =
  | typeof NameColumn
  | typeof ValidatorColumn
  | typeof LocationColumn
  | typeof ImplementationColumn
  | typeof NetworkIdColumn
  | typeof PeersColumn
  | typeof TxsColumn
  | typeof UploadColumn
  | typeof DownloadColumn
  | typeof StateCacheColumn
  | typeof BlockNumberColumn
  | typeof BlockHashColumn
  | typeof FinalizedBlockColumn
  | typeof FinalizedHashColumn
  | typeof BlockTimeColumn
  | typeof BlockPropagationColumn
  | typeof LastBlockColumn
  | typeof UptimeColumn
  | typeof CpuArchitectureColumn
  | typeof CpuColumn
  | typeof CpuCoresColumn
  | typeof LinuxDistroColumn
  | typeof LinuxKernelColumn
  | typeof IsVirtualMachineColumn
  | typeof MemoryColumn
  | typeof OperatingSystemColumn
  | typeof VersionColumn;

export interface ColumnProps {
  node: Node;
}

export function formatBytes(
  bytes: number,
  stamp: Maybe<Types.Timestamp>
): string {
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

export function formatBandwidth(
  bps: number,
  stamp: Maybe<Types.Timestamp>
): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';

  if (bps >= 1024 * 1024) {
    return `${(bps / (1024 * 1024)).toFixed(1)} MB/s${ago}`;
  } else if (bps >= 1000) {
    return `${(bps / 1024).toFixed(1)} kB/s${ago}`;
  } else {
    return `${bps | 0} B/s${ago}`;
  }
}

export const BANDWIDTH_SCALE = 1024 * 1024;

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
