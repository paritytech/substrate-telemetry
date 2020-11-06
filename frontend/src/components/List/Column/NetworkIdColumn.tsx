import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate } from '../../';
import icon from '../../../icons/fingerprint.svg';

export class NetworkIdColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Network ID';
  public static readonly icon = icon;
  public static readonly width = 90;
  public static readonly setting = 'networkId';
  public static readonly sortBy = ({ networkId }: Node) => networkId || '';

  private data: Maybe<string>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.networkId;
  }

  render() {
    const { networkId } = this.props.node;

    this.data = networkId;

    if (!networkId) {
      return <td className="Row--td">-</td>;
    }

    return (
      <td className="Row--td">
        <Truncate position="left" text={networkId} />
      </td>
    );
  }
}
