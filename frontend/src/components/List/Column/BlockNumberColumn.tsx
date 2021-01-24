import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { formatNumber } from '../../../utils';
import icon from '../../../icons/cube.svg';

export class BlockNumberColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Block';
  public static readonly icon = icon;
  public static readonly width = 88;
  public static readonly setting = 'blocknumber';
  public static readonly sortBy = ({ height }: Node) => height;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.height;
  }

  render() {
    const { height } = this.props.node;

    this.data = height;

    return <td className="Column">{`#${formatNumber(height)}`}</td>;
  }
}
