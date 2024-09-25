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

//Column for specifying whether each node runs a VM or not
import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class IsVirtualMachineColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Virtual Machine';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'is_virtual_machine';
  public static readonly sortBy = ({ is_virtual_machine }: Node) =>
    is_virtual_machine || false;

  private data: boolean;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.is_virtual_machine;
  }

  render() {
    const { is_virtual_machine } = this.props.node;

    this.data = is_virtual_machine;

    return <td className="Column">{is_virtual_machine ? 'Yes' : 'No'}</td>;
  }
}
