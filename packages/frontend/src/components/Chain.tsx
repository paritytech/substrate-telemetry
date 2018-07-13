import * as React from 'react';
import { State } from '../state';
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
    state: Readonly<State>
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

export function Chain(props: Chain.Props) {
  const { best, blockTimestamp, blockAverage } = props.state;

  const nodes = Array.from(props.state.nodes.values()).sort(sortNodes);

  return (
    <div className="Chain">
      <div className="Chain-header">
        <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
        <Tile icon={blockTimeIcon} title="Avgerage Time">{ blockAverage == null ? '-' : (blockAverage / 1000).toFixed(3) + 's' }</Tile>
        <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
      </div>
      <div className="Chain-content">
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
            nodes.map((node) => <Node key={node.id} {...node} />)
          }
          </tbody>
        </table>
      </div>
    </div>
  );
}
