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

// Each Row in the List Page
import * as React from 'react';
import { Types } from '../../common';
import { Node } from '../../state';
import { PersistentSet } from '../../persist';
import {
  Column,
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
  CpuArchitectureColumn,
  CpuColumn,
  CpuCoresColumn,
  MemoryColumn,
  OperatingSystemColumn,
  VersionColumn,
  IsVirtualMachineColumn,
  LinuxDistroColumn,
  LinuxKernelColumn,
} from './';

import './Row.css';

interface RowProps {
  node: Node;
  pins: PersistentSet<Types.NodeName>;
  columns: Column[];
}

interface RowState {
  update: number;
}

export class Row extends React.Component<RowProps, RowState> {
  public static readonly columns: Column[] = [
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
    VersionColumn,
    OperatingSystemColumn,
    CpuArchitectureColumn,
    CpuColumn,
    CpuCoresColumn,
    MemoryColumn,
    LinuxDistroColumn,
    LinuxKernelColumn,
    IsVirtualMachineColumn,
  ];
  private renderedChangeRef = 0;

  public shouldComponentUpdate(nextProps: RowProps): boolean {
    return (
      this.props.node.id !== nextProps.node.id ||
      this.renderedChangeRef !== nextProps.node.changeRef
    );
  }

  public render() {
    const { node, columns } = this.props;

    this.renderedChangeRef = node.changeRef;

    let className = 'Row';

    if (node.propagationTime != null) {
      className += ' Row-synced';
    }

    if (node.pinned) {
      className += ' Row-pinned';
    }

    if (node.stale) {
      className += ' Row-stale';
    }

    return (
      <tr className={className} onClick={this.toggle}>
        {columns.map((col, index) =>
          React.createElement(col, { node, key: index })
        )}
      </tr>
    );
  }

  public toggle = () => {
    const { pins, node } = this.props;

    if (node.pinned) {
      pins.delete(node.name);
    } else {
      pins.add(node.name);
    }
  };
}
