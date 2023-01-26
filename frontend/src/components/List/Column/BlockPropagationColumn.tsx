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
import { Maybe } from '../../../common';
import { ColumnProps } from './';
import { Node } from '../../../state';
import { milliOrSecond } from '../../../utils';
import icon from '../../../icons/dashboard.svg';

export class BlockPropagationColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Block Propagation Time';
  public static readonly icon = icon;
  public static readonly width = 58;
  public static readonly setting = 'blockpropagation';
  public static readonly sortBy = ({ propagationTime }: Node) =>
    propagationTime == null ? Infinity : propagationTime;

  private data: Maybe<number>;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.propagationTime;
  }

  render() {
    const { propagationTime } = this.props.node;
    const print =
      propagationTime == null ? '∞' : milliOrSecond(propagationTime);

    this.data = propagationTime;

    return <td className="Column">{print}</td>;
  }
}
