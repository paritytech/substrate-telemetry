// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import { Types, Maybe } from '../../common';
import { Filter } from '../';
import { State as AppState, Update as AppUpdate, Node } from '../../state';
import { Row, THead } from './';
import { Persistent, PersistentSet } from '../../persist';
import { viewport } from '../../utils';

const HEADER = 148;
const TH_HEIGHT = 35;
const TR_HEIGHT = 31;
const ROW_MARGIN = 5;

import './List.css';

interface ListProps {
  appState: Readonly<AppState>;
  appUpdate: AppUpdate;
  pins: PersistentSet<Types.NodeName>;
  sortBy: Persistent<Maybe<number>>;
}

// Helper for readability, used as `key` prop for each `Row`
// of the `List`, so that we can maximize re-using DOM elements.
type Key = number;

export class List extends React.Component<ListProps> {
  public state = {
    filter: null,
    viewportHeight: viewport().height,
  };

  private listStart = 0;
  private listEnd = 0;
  private relativeTop = -1;
  private nextKey: Key = 0;
  private previousKeys = new Map<Types.NodeId, Key>();

  public componentDidMount() {
    this.onScroll();

    window.addEventListener('resize', this.onResize);
    window.addEventListener('scroll', this.onScroll);
  }

  public componentWillUnmount() {
    window.removeEventListener('resize', this.onResize);
    window.removeEventListener('scroll', this.onScroll);
  }

  public render() {
    const { pins, sortBy, appState } = this.props;
    const { selectedColumns } = appState;
    const { filter } = this.state;

    let nodes = appState.nodes.sorted();

    if (filter != null) {
      nodes = nodes.filter(filter);

      if (nodes.length === 0) {
        return (
          <React.Fragment>
            <div className="List List-no-nodes">
              ¯\_(ツ)_/¯
              <br />
              Nothing matches
            </div>
            <Filter onChange={this.onFilterChange} />
          </React.Fragment>
        );
      }
      // With filter present, we can no longer guarantee that focus corresponds
      // to rendering view, so we put the whole list in focus
      appState.nodes.setFocus(0, nodes.length);
    } else {
      appState.nodes.setFocus(this.listStart, this.listEnd);
    }

    const height = TH_HEIGHT + nodes.length * TR_HEIGHT;
    const top = this.listStart * TR_HEIGHT;

    nodes = nodes.slice(this.listStart, this.listEnd);

    const keys = this.recalculateKeys(nodes);

    return (
      <>
        <div className="List" style={{ height }}>
          <table className="List--table">
            <THead columns={selectedColumns} sortBy={sortBy} />
            <tbody>
              <tr className="List-padding" style={{ height: `${top}px` }} />
              {nodes.map((node, i) => (
                <Row
                  key={keys[i]}
                  node={node}
                  pins={pins}
                  columns={selectedColumns}
                />
              ))}
            </tbody>
          </table>
        </div>
        <Filter onChange={this.onFilterChange} />
      </>
    );
  }

  // Get an array of keys for each `Node` in viewport in order.
  //
  // * If a `Node` was previously rendered, it will keep its `Key`.
  //
  // * If a `Node` is new to the viewport, it will get a `Key` of
  //   another `Node` that was removed from the viewport, or a new one.
  private recalculateKeys(nodes: Array<Node>): Array<Key> {
    // First we find all keys for `Node`s which didn't change from
    // last render.
    const keptKeys: Array<Maybe<Key>> = nodes.map(({ id }) => {
      const key = this.previousKeys.get(id);

      if (key != null) {
        this.previousKeys.delete(id);
      }

      return key;
    });

    // Array of all unused keys
    const unusedKeys = Array.from(this.previousKeys.values());
    let search = 0;

    // Clear the map so we can set new values
    this.previousKeys.clear();

    // Filling in blanks and re-populate previousKeys
    return keptKeys.map((key: Maybe<Key>, i) => {
      const id = nodes[i].id;

      // `Node` was previously in viewport
      if (key != null) {
        this.previousKeys.set(id, key);

        return key;
      }

      // Recycle the next unused key
      if (search < unusedKeys.length) {
        const unused = unusedKeys[search++];
        this.previousKeys.set(id, unused);

        return unused;
      }

      // No unused keys left, generate a new key
      const newKey = this.nextKey++;
      this.previousKeys.set(id, newKey);

      return newKey;
    });
  }

  private onScroll = () => {
    const relativeTop = divisibleBy(
      window.scrollY - (HEADER + TR_HEIGHT),
      TR_HEIGHT * ROW_MARGIN
    );

    if (this.relativeTop === relativeTop) {
      return;
    }

    this.relativeTop = relativeTop;

    const { viewportHeight } = this.state;
    const top = Math.max(relativeTop, 0);
    const height =
      relativeTop < 0 ? viewportHeight + relativeTop : viewportHeight;
    const listStart = Math.max(((top / TR_HEIGHT) | 0) - ROW_MARGIN, 0);
    const listEnd = listStart + ROW_MARGIN * 2 + Math.ceil(height / TR_HEIGHT);

    if (listStart !== this.listStart || listEnd !== this.listEnd) {
      this.listStart = listStart;
      this.listEnd = listEnd;
      this.props.appUpdate({});
    }
  };

  private onResize = () => {
    const viewportHeight = viewport().height;

    this.setState({ viewportHeight });
  };

  private onFilterChange = (filter: Maybe<(node: Node) => boolean>) => {
    this.setState({ filter });
  };
}

function divisibleBy(n: number, dividor: number): number {
  return n - (n % dividor);
}
