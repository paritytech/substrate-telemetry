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
import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class CpuCoresColumn extends React.Component<ColumnProps> {
  public static readonly label = 'CPU Cores';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'core_count';
  public static readonly sortBy = ({ core_count }: Node) => core_count || 0;

  private data: number;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.core_count;
  }

  render() {
    const { core_count } = this.props.node;

    this.data = core_count;

    return <td className="Column">{core_count || '-'}</td>;
  }
}
