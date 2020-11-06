import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { secondsWithPrecision } from '../../../utils';
import icon from '../../../icons/history.svg';

export class BlockTimeColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Block Time';
  public static readonly icon = icon;
  public static readonly width = 80;
  public static readonly setting = 'blocktime';
  public static readonly sortBy = ({ blockTime }: Node) =>
    blockTime == null ? Infinity : blockTime;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
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
