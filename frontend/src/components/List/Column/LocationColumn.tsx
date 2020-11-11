import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate, Tooltip } from '../../';
import icon from '../../../icons/location.svg';

export class LocationColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Location';
  public static readonly icon = icon;
  public static readonly width = 140;
  public static readonly setting = 'location';
  public static readonly sortBy = ({ city }: Node) => city || '';

  private data: Maybe<string>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.city;
  }

  render() {
    const { city } = this.props.node;

    this.data = city;

    if (!city) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Tooltip text={city} position="left" />
        <Truncate text={city} chars={14} />
      </td>
    );
  }
}
