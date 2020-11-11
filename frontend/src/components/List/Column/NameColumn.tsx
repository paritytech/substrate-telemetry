import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate, Tooltip } from '../../';
import icon from '../../../icons/server.svg';

export class NameColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Node';
  public static readonly icon = icon;
  public static readonly setting = null;
  public static readonly width = null;
  public static readonly sortBy = ({ sortableName }: Node) => sortableName;

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Node name only changes when the node does
    return this.props.node !== nextProps.node;
  }

  render() {
    const { name } = this.props.node;

    return (
      <td className="Column">
        <Tooltip text={name} position="left" />
        <Truncate text={name} />
      </td>
    );
  }
}
