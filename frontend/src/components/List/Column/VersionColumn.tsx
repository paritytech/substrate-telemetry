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

//Column for specifying which polkadot version node is running on

import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class VersionColumn extends React.Component<ColumnProps> {
  public static readonly label = 'version';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'version';
  public static readonly sortBy = ({ version }: Node) => version || '';

  private data: string;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.version;
  }

  render() {
    const { version } = this.props.node;

    this.data = version;

    return <td className="Column">{version || '-'}</td>;
  }
}
