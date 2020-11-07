import * as React from 'react';
import { Maybe } from '../../../common';
import { Column } from './';
import { Node } from '../../../state';
import { Tooltip, PolkadotIcon } from '../../';
import icon from '../../../icons/shield.svg';

export class ValidatorColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Validator';
  public static readonly icon = icon;
  public static readonly width = 16;
  public static readonly setting = 'validator';
  public static readonly sortBy = ({ validator }: Node) => validator || '';

  private data: Maybe<string>;
  private copy: Maybe<Tooltip.CopyCallback>;

  public shouldComponentUpdate(nextProps: Column.Props) {
    return this.data !== nextProps.node.validator;
  }

  render() {
    const { validator } = this.props.node;

    this.data = validator;

    if (!validator) {
      return <td className="Column">-</td>;
    }

    return (
      <td className="Column" onClick={this.onClick}>
        <Tooltip text={validator} copy={this.onCopy} />
        <PolkadotIcon
          className="Column-validator"
          account={validator}
          size={16}
        />
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
