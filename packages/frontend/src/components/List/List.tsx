import * as React from 'react';
import { Types, Maybe } from '@dotstats/common';
import { State as AppState, Node } from '../../state';
import { Row } from './';
import { PersistentSet } from '../../persist';

// const HEADER = 148;
const TH_HEIGHT = 35;
const TR_HEIGHT = 31;

import './List.css';

export namespace List {
  export interface Props {
    filter: Maybe<(node: Node) => boolean>;
    appState: Readonly<AppState>;
    pins: PersistentSet<Types.NodeName>;
  }
}

export class List extends React.Component<List.Props, {}> {
  public render() {
    const { settings } = this.props.appState;
    const { pins, filter } = this.props;
    const columns = Row.columns.filter(({ setting }) => setting == null || settings[setting]);

    let nodes = this.props.appState.nodes.sorted();

    if (filter != null) {
      nodes = nodes.filter(filter);

      if (nodes.length === 0) {
        return (
          <div className="List List-no-nodes">¯\_(ツ)_/¯<br />Nothing matches</div>
        );
      }
    }

    const height = TH_HEIGHT + nodes.length * TR_HEIGHT;

    return (
      <div className="List" style={{ height }}>
        <table>
          <Row.Header columns={columns} />
          <tbody>
          {
            nodes.map((node) => <Row key={node.id} node={node} pins={pins} columns={columns} />)
          }
          </tbody>
        </table>
      </div>
    );
  }
}
