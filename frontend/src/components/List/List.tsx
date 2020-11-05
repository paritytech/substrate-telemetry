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
const FIREFOX = /Firefox/i.test(navigator.userAgent);

import './List.css';

export namespace List {
  export interface Props {
    appState: Readonly<AppState>;
    appUpdate: AppUpdate;
    pins: PersistentSet<Types.NodeName>;
    sortBy: Persistent<Maybe<number>>;
  }

  export interface State {
    filter: Maybe<(node: Node) => boolean>;
    viewportHeight: number;
  }
}

type Key = number;

export class List extends React.Component<List.Props, {}> {
  public state = {
    filter: null,
    viewportHeight: viewport().height,
  };

  private listStart = 0;
  private listEnd = 0;
  private nextKey: Key = 0;
  private relativeTop = -1;
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

    // Firefox supports relative positions for table elements but suffers badly
    // when doing translate.
    // Chrome doesn't support relative positions, but renders translates without issues.
    const tbodyStyle = FIREFOX
      ? { top: `${top}px` }
      : { transform: `translateY(${top}px)` };

    nodes = nodes.slice(this.listStart, this.listEnd);

    const keys: Array<Maybe<Key>> = nodes.map((node) => {
      const key = this.previousKeys.get(node.id);

      if (key) {
        this.previousKeys.delete(node.id);
        return key;
      } else {
        return null;
      }
    });

    const unusedKeys = Array.from(this.previousKeys.values());

    let search = 0;

    const nextUnusedKey = () => {
      if (search < unusedKeys.length) {
        return unusedKeys[search++];
      } else {
        return this.nextKey++;
      }
    };

    this.previousKeys.clear();

    return (
      <React.Fragment>
        <div className="List" style={{ height }}>
          <table>
            <THead columns={selectedColumns} sortBy={sortBy} />
            <tbody style={tbodyStyle}>
              {nodes.map((node, i) => {
                const newKey = (keys[i] || nextUnusedKey()) as number;

                this.previousKeys.set(node.id, newKey);

                return (
                  <Row
                    key={newKey}
                    node={node}
                    pins={pins}
                    columns={selectedColumns}
                  />
                );
              })}
            </tbody>
          </table>
        </div>
        <Filter onChange={this.onFilterChange} />
      </React.Fragment>
    );
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
