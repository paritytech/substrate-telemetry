import * as React from 'react';
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
  | typeof UptimeColumn;

export namespace Column {
  export interface Props {
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
