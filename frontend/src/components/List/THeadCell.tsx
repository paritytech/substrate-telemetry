import * as React from 'react';
import { Maybe } from '../../common';
import { Column } from './';
import { Icon, Tooltip } from '../';
import { Persistent } from '../../persist';

import sortAscIcon from '../../icons/triangle-up.svg';
import sortDescIcon from '../../icons/triangle-down.svg';

export namespace THeadCell {
  export interface Props {
    column: Column;
    index: number;
    last: number;
    sortBy: Persistent<Maybe<number>>;
  }
}

export class THeadCell extends React.Component<THeadCell.Props, {}> {
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
