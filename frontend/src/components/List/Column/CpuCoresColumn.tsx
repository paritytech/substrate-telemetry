import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class CpuCoresColumn extends React.Component<ColumnProps> {
    public static readonly label = 'CPU Cores';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'core_count';
    public static readonly sortBy = ({ core_count }: Node) => core_count || 0;
  
    private data: number;
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.core_count;
    }
  
    render() {
      const { core_count } = this.props.node;
  
      this.data = core_count;
  
      return (
        <td className="Column">
          {core_count}
        </td>
      );
    }
  }
  