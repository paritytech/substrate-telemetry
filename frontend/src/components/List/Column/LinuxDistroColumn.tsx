import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class LinuxDistroColumn extends React.Component<ColumnProps> {
    public static readonly label = 'Linux Distro';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'linux_distro';
    public static readonly sortBy = ({ linux_distro }: Node) => linux_distro || '';
  
    private data: string;
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.linux_distro;
    }
  
    render() {
      const { linux_distro } = this.props.node;
  
      this.data = linux_distro;
  
      return (
        <td className="Column">
          {linux_distro}
        </td>
      );
    }
  }
