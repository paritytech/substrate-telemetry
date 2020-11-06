import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { Ago } from '../../';
import icon from '../../../icons/watch.svg';

export class LastBlockColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Last Block Time';
  public static readonly icon = icon;
  public static readonly width = 100;
  public static readonly setting = 'blocklasttime';
  public static readonly sortBy = ({ blockTimestamp }: Node) =>
    blockTimestamp || 0;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.blockTimestamp;
  }

  render() {
    const { blockTimestamp } = this.props.node;

    this.data = blockTimestamp;

    return (
      <td className="Column">
        <Ago when={blockTimestamp} />
      </td>
    );
  }
}
