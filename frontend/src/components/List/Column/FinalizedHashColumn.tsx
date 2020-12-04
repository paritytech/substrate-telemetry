import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate, Tooltip } from '../../';
import icon from '../../../icons/file-binary.svg';

export class FinalizedHashColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Finalized Block Hash';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'finalizedhash';
  public static readonly sortBy = ({ finalizedHash }: Node) =>
    finalizedHash || '';

  private data: Maybe<string>;
  private copy: Maybe<Tooltip.CopyCallback>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.finalizedHash;
  }

  render() {
    const { finalizedHash } = this.props.node;

    this.data = finalizedHash;

    return (
      <td className="Column" onClick={this.onClick}>
        <Tooltip text={finalizedHash} position="right" copy={this.onCopy} />
        <Truncate text={finalizedHash} chars={16} />
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
