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
import { Truncate, Tooltip } from '../../';
import icon from '../../../icons/server.svg';

export class NameColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Node';
  public static readonly icon = icon;
  public static readonly setting = null;
  public static readonly width = null;
  public static readonly sortBy = ({ sortableName }: Node) => sortableName;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    // Node name only changes when the node does
    return this.props.node !== nextProps.node;
  }

  render() {
    const { name } = this.props.node;

    return (
      <td className="Column">
        <Tooltip text={name} position="left" />
        <Truncate text={name} />
      </td>
    );
  }
}
