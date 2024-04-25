import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class LinuxKernelColumn extends React.Component<ColumnProps> {
    public static readonly label = 'Linux Kernel';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'linux_kernel';
    public static readonly sortBy = ({ linux_kernel }: Node) => linux_kernel || 0;
  
    private data: number;
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.linux_kernel;
    }
  
    render() {
      const { linux_kernel } = this.props.node;
  
      this.data = linux_kernel;
  
      return (
        <td className="Column">
          {linux_kernel}
        </td>
      );
    }
  }
