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
import icon from '../../../icons/broadcast.svg';

export class PeersColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Peer Count';
  public static readonly icon = icon;
  public static readonly width = 26;
  public static readonly setting = 'peers';
  public static readonly sortBy = ({ peers }: Node) => peers;

  private data = 0;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.peers;
  }

  render() {
    const { peers } = this.props.node;

    this.data = peers;

    return <td className="Column">{peers}</td>;
  }
}
