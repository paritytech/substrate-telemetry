import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class MemoryColumn extends React.Component<ColumnProps> {
    public static readonly label = 'memory';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'memory';
    public static readonly sortBy = ({ memory }: Node) => memory|| '';
  
    private data: number;

  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.memory;
    }
  
    render() {
      const { memory} = this.props.node;
  
      this.data = memory;
  
      return (
        <td className="Column">
          {memory} 
        </td>
      );
    }
  }