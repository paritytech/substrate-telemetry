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
import { secondsWithPrecision } from '../../../utils';
import icon from '../../../icons/history.svg';

export class BlockTimeColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Block Time';
  public static readonly icon = icon;
  public static readonly width = 80;
  public static readonly setting = 'blocktime';
  public static readonly sortBy = ({ blockTime }: Node) =>
    blockTime == null ? Infinity : blockTime;

  private data = 0;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.blockTime;
  }

  render() {
    const { blockTime } = this.props.node;

    this.data = blockTime;

    return (
      <td className="Column">{`${secondsWithPrecision(blockTime / 1000)}`}</td>
    );
  }
}
