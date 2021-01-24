import * as React from 'react';
import { Maybe } from '../../common';
import { Column, THeadCell } from './';
import { Persistent } from '../../persist';

import './THead.css';

export namespace THead {
  export interface Props {
    columns: Column[];
    sortBy: Persistent<Maybe<number>>;
  }
}

export class THead extends React.Component<THead.Props, {}> {
  private sortBy: Maybe<number>;

  constructor(props: THead.Props) {
    super(props);

    this.sortBy = props.sortBy.get();
  }

  public shouldComponentUpdate(nextProps: THead.Props) {
    return this.sortBy !== nextProps.sortBy.get();
  }

  public render() {
    const { columns, sortBy } = this.props;
    const last = columns.length - 1;

    this.sortBy = sortBy.get();

    return (
      <thead>
        <tr className="THead">
          {columns.map((col, index) => (
            <THeadCell
              key={index}
              column={col}
              index={index}
              last={last}
              sortBy={sortBy}
            />
          ))}
        </tr>
      </thead>
    );
  }
}
