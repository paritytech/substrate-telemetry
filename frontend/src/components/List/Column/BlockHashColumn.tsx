import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate, Tooltip } from '../../';
import icon from '../../../icons/file-binary.svg';

export class BlockHashColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Block Hash';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'blockhash';
  public static readonly sortBy = ({ hash }: Node) => hash || '';

  private data: Maybe<string>;
  private copy: Maybe<Tooltip.CopyCallback>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.hash;
  }

  render() {
    const { hash } = this.props.node;

    this.data = hash;

    return (
      <td className="Column" onClick={this.onClick}>
        <Tooltip text={hash} position="right" copy={this.onCopy} />
        <Truncate text={hash} chars={16} />
      </td>
    );
  }

  private onCopy = (copy: Tooltip.CopyCallback) => {
    this.copy = copy;
  };

  private onClick = (event: React.MouseEvent) => {
    event.stopPropagation();

    if (this.copy != null) {
      this.copy();
    }
  };
}
