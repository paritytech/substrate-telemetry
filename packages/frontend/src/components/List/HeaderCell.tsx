import * as React from 'react';
import { Column } from './';
import { Icon, Tooltip } from '../';

export namespace HeaderCell {
  export interface Props {
    column: Column;
    first: boolean;
    last: boolean;
  }
}

export class HeaderCell extends React.Component<HeaderCell.Props, {}> {
  public render() {
    const { column, first, last } = this.props;
    const { icon, width, label } = column;
    const position = first ? 'left'
                   : last ? 'right'
                   : 'center';

    return (
      <th style={width ? { width } : undefined} onClick={this.toggleSort}>
        <Tooltip text={label} inline={true} position={position}><Icon src={icon} /></Tooltip>
      </th>
    )
  }

  private toggleSort = () => {
    console.log('toggle sort', this.props.column.label);
  }
}
