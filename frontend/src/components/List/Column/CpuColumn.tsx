import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class CpuColumn extends React.Component<ColumnProps> {
    public static readonly label = 'CPU Column';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'cpu';
    public static readonly sortBy = ({ cpu }: Node) => cpu || 0;
  
    private data: number;
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.cpu;
    }

    render() {
      const { cpu } = this.props.node;
  
      this.data = cpu;

      return (
        <td className="Column">
          {cpu}
        </td>
      ); 
      
    }
  
    
  }
  