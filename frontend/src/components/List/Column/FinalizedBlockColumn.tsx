import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { formatNumber } from '../../../utils';
import icon from '../../../icons/cube-alt.svg';

export class FinalizedBlockColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Finalized Block';
  public static readonly icon = icon;
  public static readonly width = 88;
  public static readonly setting = 'finalized';
  public static readonly sortBy = ({ finalized }: Node) => finalized || 0;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.finalized;
  }

  render() {
    const { finalized } = this.props.node;

    this.data = finalized;

    return <td className="Column">{`#${formatNumber(finalized)}`}</td>;
  }
}
