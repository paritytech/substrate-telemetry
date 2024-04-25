import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class OperatingSystemColumn extends React.Component<ColumnProps> {
    public static readonly label = 'OS';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'target_os';
    public static readonly sortBy = ({ target_os }: Node) => target_os || '';
  
    private data: string;
    
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.hash;
    }
  
    render() {
      const { target_os } = this.props.node;
  
      this.data = target_os;
  
      return (
        <td className="Column">
          {target_os} 
        </td>
      );
    }
  }