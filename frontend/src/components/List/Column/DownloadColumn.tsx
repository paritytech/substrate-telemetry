import * as React from 'react';
import { Types, Maybe, timestamp } from '../../../common';
import { Column, BANDWIDTH_SCALE } from './';
import { Node } from '../../../state';
import { Sparkline } from '../../';
import icon from '../../../icons/cloud-download.svg';

export class DownloadColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Download Bandwidth';
  public static readonly icon = icon;
  public static readonly width = 40;
  public static readonly setting = 'download';
  public static readonly sortBy = ({ download }: Node) =>
    download.length < 3 ? 0 : download[download.length - 1];

  private data: Array<number> = [];

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Diffing by ref, as data is an immutable array
    return this.data !== nextProps.node.download;
  }

  render() {
    const { download, chartstamps } = this.props.node;

    this.data = download;

    if (download.length < 3) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={Column.formatBandwidth}
          values={download}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      </td>
    );
  }
}
