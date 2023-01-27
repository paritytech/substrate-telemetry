// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import { ColumnProps, formatBandwidth, BANDWIDTH_SCALE } from './';
import { Node } from '../../../state';
import { Sparkline } from '../../';
import icon from '../../../icons/cloud-upload.svg';

export class UploadColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Upload Bandwidth';
  public static readonly icon = icon;
  public static readonly width = 40;
  public static readonly setting = 'upload';
  public static readonly sortBy = ({ upload }: Node) =>
    upload.length < 3 ? 0 : upload[upload.length - 1];

  private data: Array<number> = [];

  public shouldComponentUpdate(nextProps: ColumnProps) {
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
          format={formatBandwidth}
          values={upload}
          stamps={chartstamps}
          minScale={BANDWIDTH_SCALE}
        />
      </td>
    );
  }
}
