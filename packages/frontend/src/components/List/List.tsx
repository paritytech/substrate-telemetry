import * as React from 'react';
import { Types, Maybe } from '@dotstats/common';
import { Filter } from '../';
import { State as AppState, Node } from '../../state';
import { Row } from './';
import { Persistent, PersistentSet } from '../../persist';
import { viewport } from '../../utils'

const HEADER = 148;
const TH_HEIGHT = 35;
const TR_HEIGHT = 31;
const ROW_MARGIN = 5;

import './List.css';

export namespace List {
  export interface Props {
    appState: Readonly<AppState>;
    pins: PersistentSet<Types.NodeName>;
    sortBy: Persistent<Maybe<number>>;
  }

  export interface State {
    filter: Maybe<(node: Node) => boolean>;
    viewportHeight: number;
    listStart: number;
    listEnd: number;
  }
}

export class List extends React.Component<List.Props, {}> {
  public state = {
    filter: null,
    viewportHeight: viewport().height,
    listStart: 0,
    listEnd: 0,
  };

  private relativeTop = -1;
  private scrolling = false;

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
    const { selectedColumns } = this.props.appState;
    const { pins, sortBy } = this.props;
    const { filter } = this.state;

    let nodes = this.props.appState.nodes.sorted();

    if (filter != null) {
      nodes = nodes.filter(filter);

      if (nodes.length === 0) {
        return (
          <React.Fragment>
            <div className="List List-no-nodes">¯\_(ツ)_/¯<br />Nothing matches</div>
            <Filter onChange={this.onFilterChange} />
          </React.Fragment>
        );
      }
    }

    const { listStart, listEnd } = this.state;

    const height = (TH_HEIGHT + nodes.length * TR_HEIGHT);
    const transform = `translateY(${listStart * TR_HEIGHT}px)`;

    nodes = nodes.slice(listStart, listEnd);

    return (
      <React.Fragment>
        <div className="List" style={{ height }}>
          <table>
            <Row.Header columns={selectedColumns} sortBy={sortBy} />
            <tbody style={{ transform }}>
            {
              nodes.map((node) => <Row key={node.id} node={node} pins={pins} columns={selectedColumns} />)
            }
            </tbody>
          </table>
        </div>
        <Filter onChange={this.onFilterChange} />
      </React.Fragment>
    );
  }

  private onScroll = () => {
    if (this.scrolling) {
      return;
    }

    const relativeTop = divisibleBy(window.scrollY - (HEADER + TR_HEIGHT), TR_HEIGHT * ROW_MARGIN);

    if (this.relativeTop === relativeTop) {
      return;
    }

    this.relativeTop = relativeTop;
    this.scrolling = true;

    window.requestAnimationFrame(this.onScrollRAF);
  }

  private onScrollRAF = () => {
    const { relativeTop } = this;
    const { viewportHeight } = this.state;
    const top = Math.max(relativeTop, 0);
    const height = relativeTop < 0 ? viewportHeight + relativeTop : viewportHeight;
    const listStart = Math.max((top / TR_HEIGHT | 0) - ROW_MARGIN, 0);
    const listEnd = listStart + ROW_MARGIN * 2 + Math.ceil(height / TR_HEIGHT);

    if (listStart !== this.state.listStart || listEnd !== this.state.listEnd) {
      this.setState({ listStart, listEnd });
    }

    this.scrolling = false;
  }

  private onResize = () => {
    const viewportHeight = viewport().height;

    this.setState({ viewportHeight });
  }

  private onFilterChange = (filter: Maybe<(node: Node) => boolean>) => {
    this.setState({ filter });
  }
}

function divisibleBy(n: number, dividor: number): number {
  return n - n % dividor;
}
