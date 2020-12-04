import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import icon from '../../../icons/broadcast.svg';

export class PeersColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Peer Count';
  public static readonly icon = icon;
  public static readonly width = 26;
  public static readonly setting = 'peers';
  public static readonly sortBy = ({ peers }: Node) => peers;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.peers;
  }

  render() {
    const { peers } = this.props.node;

    this.data = peers;

    return <td className="Column">{peers}</td>;
  }
}
