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
import { Maybe } from '../../common';
import { Column } from './';
import { Icon, Tooltip } from '../';
import { Persistent } from '../../persist';

import sortAscIcon from '../../icons/triangle-up.svg';
import sortDescIcon from '../../icons/triangle-down.svg';

interface THeadCellProps {
  column: Column;
  index: number;
  last: number;
  sortBy: Persistent<Maybe<number>>;
}

export class THeadCell extends React.Component<THeadCellProps> {
  public render() {
    const { column, index, last } = this.props;
    const { icon, width, label } = column;
    const position = index === 0 ? 'left' : index === last ? 'right' : 'center';

    const sortBy = this.props.sortBy.get();
    const className =
      column.sortBy == null
        ? 'THeadCell'
        : sortBy === index || sortBy === ~index
        ? 'THeadCell THeadCell-sorted'
        : 'THeadCell THeadCell-sortable';
    const i =
      sortBy === index ? sortAscIcon : sortBy === ~index ? sortDescIcon : icon;

    return (
      <th
        className={className}
        style={width ? { width } : undefined}
        onClick={this.toggleSort}
      >
        <span className="THeadCell-container">
          <Tooltip text={label} position={position} />
          <Icon src={i} />
        </span>
      </th>
    );
  }

  private toggleSort = () => {
    const { index, sortBy, column } = this.props;
    const sortByRaw = sortBy.get();

    if (column.sortBy == null) {
      return;
    }

    if (sortByRaw === index) {
      sortBy.set(~index);
    } else if (sortByRaw === ~index) {
      sortBy.set(null);
    } else {
      sortBy.set(index);
    }
  };
}
