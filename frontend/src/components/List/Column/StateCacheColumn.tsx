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
import { ColumnProps, formatBytes, BANDWIDTH_SCALE } from './';
import { Node } from '../../../state';
import { Sparkline } from '../../';
import icon from '../../../icons/git-branch.svg';

export class StateCacheColumn extends React.Component<ColumnProps> {
  public static readonly label = 'State Cache Size';
  public static readonly icon = icon;
  public static readonly width = 40;
  public static readonly setting = 'stateCacheSize';
  public static readonly sortBy = ({ stateCacheSize }: Node) =>
    stateCacheSize.length < 3 ? 0 : stateCacheSize[stateCacheSize.length - 1];

  private data: Array<number> = [];

  public shouldComponentUpdate(nextProps: ColumnProps) {
    // Diffing by ref, as data is an immutable array
    return this.data !== nextProps.node.stateCacheSize;
  }

  render() {
    const { stateCacheSize, chartstamps } = this.props.node;

    this.data = stateCacheSize;

    if (stateCacheSize.length < 3) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={formatBytes}
          values={stateCacheSize}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      </td>
    );
  }
}
