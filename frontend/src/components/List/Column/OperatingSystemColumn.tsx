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

//Column for specifying which OS the node is running on

import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class OperatingSystemColumn extends React.Component<ColumnProps> {
  public static readonly label = 'OS';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'target_os';
  public static readonly sortBy = ({ target_os }: Node) => target_os || '';

  private data: string;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.hash;
  }

  render() {
    const { target_os } = this.props.node;

    this.data = target_os;

    return <td className="Column">{target_os || '-'}</td>;
  }
}
