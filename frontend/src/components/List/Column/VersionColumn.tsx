import * as React from 'react';
import { ColumnProps } from './';
import { Node } from '../../../state';
import icon from '../../../icons/file-binary.svg';

export class VersionColumn extends React.Component<ColumnProps> {
    public static readonly label = 'version';
    public static readonly icon = icon;
    public static readonly width = 154;
    public static readonly setting = 'version';
    public static readonly sortBy = ({ version}: Node) => version || '';
  
    private data: string;
    
  
    public shouldComponentUpdate(nextProps: ColumnProps) {
      return this.data !== nextProps.node.version;
    }
  
    render() {
      const { version } = this.props.node;
  
      this.data = version;
  
      return (
        <td className="Column" >
         {version}
        </td>
      );
    }
  }