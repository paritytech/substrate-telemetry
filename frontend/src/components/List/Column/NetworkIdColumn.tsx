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
import { Maybe } from '../../../common';
import { ColumnProps } from './';
import { Node } from '../../../state';
import { Tooltip, TooltipCopyCallback } from '../../';
import icon from '../../../icons/fingerprint.svg';

export class NetworkIdColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Network ID';
  public static readonly icon = icon;
  public static readonly width = 90;
  public static readonly setting = 'networkId';
  public static readonly sortBy = ({ networkId }: Node) => networkId || '';

  private data: Maybe<string>;
  private copy: Maybe<TooltipCopyCallback>;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.networkId;
  }

  render() {
    const { networkId } = this.props.node;

    this.data = networkId;

    if (!networkId) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column" onClick={this.onClick}>
        <Tooltip text={networkId} position="left" copy={this.onCopy} />
        {networkId}
      </td>
    );
  }

  private onCopy = (copy: TooltipCopyCallback) => {
    this.copy = copy;
  };

  private onClick = (event: React.MouseEvent) => {
    event.stopPropagation();

    if (this.copy != null) {
      this.copy();
    }
  };
}
