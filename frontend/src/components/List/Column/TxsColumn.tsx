import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import icon from '../../../icons/inbox.svg';

export class TxsColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Transactions in Queue';
  public static readonly icon = icon;
  public static readonly width = 26;
  public static readonly setting = 'txs';
  public static readonly sortBy = ({ txs }: Node) => txs;

  private data = 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.txs;
  }

  render() {
    const { txs } = this.props.node;

    this.data = txs;

    return <td className="Column">{txs}</td>;
  }
}
