import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class CpuArchitectureColumn extends React.Component<ColumnProps> {
    public static readonly label = 'CPU Architecture';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'target_arch';
    public static readonly sortBy = ({ target_arch }: Node) => target_arch || '';
  
    private data: string;
    
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.hash;
    }
  
    render() {
      const { target_arch } = this.props.node;
  
      this.data = target_arch;
  
      return (
        <td className="Column">
          {target_arch} 
        </td>
      );
    }
  }

  