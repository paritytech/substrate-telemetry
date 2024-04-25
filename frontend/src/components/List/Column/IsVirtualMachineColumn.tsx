import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class IsVirtualMachineColumn extends React.Component<ColumnProps> {
    public static readonly label = 'Virtual Machine';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'is_virtual_machine';
    public static readonly sortBy = ({ is_virtual_machine }: Node) => is_virtual_machine || false;
  
    private data: boolean;

  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.is_virtual_machine;
    }
  
    render() {
      const { is_virtual_machine } = this.props.node;
  
      this.data = is_virtual_machine;
  
      return (
        <td className="Column">
          {is_virtual_machine}
        </td>
      );
    }
  }