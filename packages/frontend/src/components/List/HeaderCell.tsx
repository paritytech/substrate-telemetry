import * as React from 'react';
import { Maybe } from '@dotstats/common';
import { Column } from './';
import { Icon, Tooltip } from '../';
import { Persistent } from '../../persist';

import sortAscIcon from '../../icons/triangle-up.svg';
import sortDescIcon from '../../icons/triangle-down.svg';

export namespace HeaderCell {
  export interface Props {
    column: Column;
    index: number;
    last: number;
    sortBy: Persistent<Maybe<number>>;
  }
}

export class HeaderCell extends React.Component<HeaderCell.Props, {}> {
  public render() {
    const { column, index, last } = this.props;
    const { icon, width, label } = column;
    const position = index === 0 ? 'left'
                   : index === last ? 'right'
                   : 'center';

    const sortBy = this.props.sortBy.get();
    const className = column.sortBy == null ? '' : sortBy === index || sortBy === ~index ? 'HeaderCell-sorted' : 'HeaderCell-sortable';
    const i = sortBy === index ? sortAscIcon : sortBy === ~index ? sortDescIcon : icon;

    return (
      <th className={className} style={width ? { width } : undefined} onClick={this.toggleSort}>
        <Tooltip text={label} inline={true} position={position}><Icon src={i} /></Tooltip>
      </th>
    )
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
  }
}
