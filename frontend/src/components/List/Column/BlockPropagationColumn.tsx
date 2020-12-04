import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { milliOrSecond } from '../../../utils';
import icon from '../../../icons/dashboard.svg';

export class BlockPropagationColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Block Propagation Time';
  public static readonly icon = icon;
  public static readonly width = 58;
  public static readonly setting = 'blockpropagation';
  public static readonly sortBy = ({ propagationTime }: Node) =>
    propagationTime == null ? Infinity : propagationTime;

  private data: Maybe<number>;

  public shouldComponentUpdate(nextProps: Column.Props) {
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
