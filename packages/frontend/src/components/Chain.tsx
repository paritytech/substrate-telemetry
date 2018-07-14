import * as React from 'react';
import { State as AppState } from '../state';
import { formatNumber } from '../utils';
import { Tile, Icon, Node, Ago } from './';

import nodeIcon from '../icons/server.svg';
import nodeTypeIcon from '../icons/terminal.svg';
import peersIcon from '../icons/broadcast.svg';
import transactionsIcon from '../icons/inbox.svg';
import blockIcon from '../icons/package.svg';
import blockHashIcon from '../icons/file-binary.svg';
import blockTimeIcon from '../icons/history.svg';
import propagationTimeIcon from '../icons/dashboard.svg';
import lastTimeIcon from '../icons/watch.svg';

import './Chain.css';

export namespace Chain {
  export interface Props {
    appState: Readonly<AppState>;
  }

  export interface State {
    display: 'map' | 'table';
  }
}

function sortNodes(a: Node.Props, b: Node.Props): number {
  const aPropagation = a.blockDetails[4] == null ? Infinity : a.blockDetails[4] as number;
  const bPropagation = b.blockDetails[4] == null ? Infinity : b.blockDetails[4] as number;

  if (aPropagation === Infinity && bPropagation === Infinity) {
    // Descending sort by block number
    return b.blockDetails[0] - a.blockDetails[0];
  }

  // Ascending sort by propagation time
  return aPropagation - bPropagation;
}

export class Chain extends React.Component<Chain.Props, Chain.State> {
  constructor(props: Chain.Props) {
    super(props);

    this.state = {
      display: 'table'
    };
  }

  public render() {
    const { best, blockTimestamp, blockAverage } = this.props.appState;

    return (
      <div className="Chain">
        <div className="Chain-header">
          <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
          <Tile icon={blockTimeIcon} title="Avgerage Time">{ blockAverage == null ? '-' : (blockAverage / 1000).toFixed(3) + 's' }</Tile>
          <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
        </div>
        <div className="Chain-content-container">
          <div className="Chain-content">
          {
            this.state.display === 'table'
              ? this.renderTable()
              : this.renderMap()
          }
          </div>
        </div>
      </div>
    );
  }

  private renderMap() {
    // return <ReactSVG path={worldMap} className="Chain-map" />;
    return (
      <div className="Chain-map">
      {
        this.nodes().map((node) => <div key={node.id} className="Chain-map-node" data-foo={JSON.stringify(node)} />)
      }
      </div>
    );
  }

  private renderTable() {
    return (
      <table className="Chain-node-list">
        <thead>
          <tr>
            <th><Icon src={nodeIcon} alt="Node" /></th>
            <th><Icon src={nodeTypeIcon} alt="Implementation" /></th>
            <th><Icon src={peersIcon} alt="Peer Count" /></th>
            <th><Icon src={transactionsIcon} alt="Transactions in Queue" /></th>
            <th><Icon src={blockIcon} alt="Block" /></th>
            <th><Icon src={blockHashIcon} alt="Block Hash" /></th>
            <th><Icon src={blockTimeIcon} alt="Block Time" /></th>
            <th><Icon src={propagationTimeIcon} alt="Block Propagation Time" /></th>
            <th><Icon src={lastTimeIcon} alt="Last Block Time" /></th>
          </tr>
        </thead>
        <tbody>
        {
          this.nodes().sort(sortNodes).map((node) => <Node key={node.id} {...node} />)
        }
        </tbody>
      </table>
    );
  }

  private nodes() {
    return Array.from(this.props.appState.nodes.values());
  }
}
