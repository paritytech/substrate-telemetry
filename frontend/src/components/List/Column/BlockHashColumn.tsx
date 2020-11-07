import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate } from '../';
import icon from '../../../icons/file-binary.svg';

export class BlockHashColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Block Hash';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'blockhash';
  public static readonly sortBy = ({ hash }: Node) => hash || '';

  private data: Maybe<string>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.hash;
  }

  render() {
    const { hash } = this.props.node;

    this.data = hash;

    return (
      <td className="Column">
        <Truncate text={hash} chars={16} position="right" copy={true} />
      </td>
    );
  }
}
