import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { Ago } from '../../';
import icon from '../../../icons/pulse.svg';

export class UptimeColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Node Uptime';
  public static readonly icon = icon;
  public static readonly width = 58;
  public static readonly setting = 'uptime';
  public static readonly sortBy = ({ startupTime }: Node) => startupTime || 0;

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Uptime only changes when the node does
    return this.props.node !== nextProps.node;
  }

  render() {
    const { startupTime } = this.props.node;

    if (!startupTime) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Ago when={startupTime} justTime={true} />
      </td>
    );
  }
}
