import * as React from 'react';
import { Types, Maybe } from '../../common';
import { Node } from '../../state';
import { Persistent, PersistentSet } from '../../persist';
import { Column } from './';

import './Row.css';

export namespace Row {
  export interface Props {
    node: Node;
    pins: PersistentSet<Types.NodeName>;
    columns: Column[];
  }

  export interface State {
    update: number;
  }
}

interface HeaderProps {
  columns: Column[];
  sortBy: Persistent<Maybe<number>>;
}

export class Row extends React.Component<Row.Props, Row.State> {
  public static readonly columns: Column[] = [
    Column.NAME,
    Column.VALIDATOR,
    Column.LOCATION,
    Column.IMPLEMENTATION,
    Column.NETWORK_ID,
    Column.PEERS,
    Column.TXS,
    Column.UPLOAD,
    Column.DOWNLOAD,
    Column.STATE_CACHE,
    Column.BLOCK_NUMBER,
    Column.BLOCK_HASH,
    Column.FINALIZED,
    Column.FINALIZED_HASH,
    Column.BLOCK_TIME,
    Column.BLOCK_PROPAGATION,
    Column.BLOCK_LAST_TIME,
    Column.UPTIME,
    Column.NETWORK_STATE,
  ];

  // public static HEADER = (props: HeaderProps) => {
  //   const { columns, sortBy } = props;
  //   const last = columns.length - 1;

  //   return (
  //     <thead>
  //       <tr className="Row-Header">
  //         {columns.map((col, index) => (
  //           <HeaderCell
  //             key={index}
  //             column={col}
  //             index={index}
  //             last={last}
  //             sortBy={sortBy}
  //           />
  //         ))}
  //       </tr>
  //     </thead>
  //   );
  // };

  private renderedChangeRef = 0;

  public shouldComponentUpdate(nextProps: Row.Props): boolean {
    return (
      this.props.node.id !== nextProps.node.id ||
      this.renderedChangeRef !== nextProps.node.changeRef
    );
  }

  public render() {
    const { node, columns } = this.props;

    this.renderedChangeRef = node.changeRef;

    let className = 'Row';

    if (node.propagationTime != null) {
      className += ' Row-synced';
    }

    if (node.pinned) {
      className += ' Row-pinned';
    }

    if (node.stale) {
      className += ' Row-stale';
    }

    return (
      <tr className={className} onClick={this.toggle}>
        {columns.map(({ render }, index) => (
          <td key={index}>{render(node)}</td>
        ))}
      </tr>
    );
  }

  public toggle = () => {
    const { pins, node } = this.props;

    if (node.pinned) {
      pins.delete(node.name);
    } else {
      pins.add(node.name);
    }
  };
}
