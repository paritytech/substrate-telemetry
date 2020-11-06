import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Truncate } from '../';
import icon from '../../../icons/file-binary.svg';

export class FinalizedHashColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Finalized Block Hash';
  public static readonly icon = icon;
  public static readonly width = 154;
  public static readonly setting = 'finalizedhash';
  public static readonly sortBy = ({ finalizedHash }: Node) =>
    finalizedHash || '';

  private data: Maybe<string>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.finalizedHash;
  }

  render() {
    const { finalizedHash } = this.props.node;

    this.data = finalizedHash;

    return (
      <td className="Column">
        <Truncate text={finalizedHash} position="right" copy={true} />
      </td>
    );
  }
}
