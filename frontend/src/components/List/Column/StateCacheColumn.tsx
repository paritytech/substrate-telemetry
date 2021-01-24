import * as React from 'react';
import { Types, Maybe, timestamp } from '../../../common';
import { Column, BANDWIDTH_SCALE } from './';
import { Node } from '../../../state';
import { Sparkline } from '../../';
import icon from '../../../icons/git-branch.svg';

export class StateCacheColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'State Cache Size';
  public static readonly icon = icon;
  public static readonly width = 40;
  public static readonly setting = 'stateCacheSize';
  public static readonly sortBy = ({ stateCacheSize }: Node) =>
    stateCacheSize.length < 3 ? 0 : stateCacheSize[stateCacheSize.length - 1];

  private data: Array<number> = [];

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Diffing by ref, as data is an immutable array
    return this.data !== nextProps.node.stateCacheSize;
  }

  render() {
    const { stateCacheSize, chartstamps } = this.props.node;

    this.data = stateCacheSize;

    if (stateCacheSize.length < 3) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={Column.formatBytes}
          values={stateCacheSize}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      </td>
    );
  }
}
