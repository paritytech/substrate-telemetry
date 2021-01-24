import * as React from 'react';
import { Types, Maybe, timestamp } from '../../../common';
import { Column, BANDWIDTH_SCALE } from './';
import { Node } from '../../../state';
import { Sparkline } from '../../';
import icon from '../../../icons/cloud-upload.svg';

export class UploadColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Upload Bandwidth';
  public static readonly icon = icon;
  public static readonly width = 40;
  public static readonly setting = 'upload';
  public static readonly sortBy = ({ upload }: Node) =>
    upload.length < 3 ? 0 : upload[upload.length - 1];

  private data: Array<number> = [];

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Diffing by ref, as data is an immutable array
    return this.data !== nextProps.node.upload;
  }

  render() {
    const { upload, chartstamps } = this.props.node;

    this.data = upload;

    if (upload.length < 3) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column">
        <Sparkline
          width={44}
          height={16}
          stroke={1}
          format={Column.formatBandwidth}
          values={upload}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      </td>
    );
  }
}
