import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../utils';
import { Ago, Icon } from './';
import { Types, Maybe } from '@dotstats/common';

import nodeIcon from '../icons/server.svg';
import nodeTypeIcon from '../icons/terminal.svg';
import peersIcon from '../icons/broadcast.svg';
import transactionsIcon from '../icons/inbox.svg';
import blockIcon from '../icons/package.svg';
import blockHashIcon from '../icons/file-binary.svg';
import blockTimeIcon from '../icons/history.svg';
import propagationTimeIcon from '../icons/dashboard.svg';
import lastTimeIcon from '../icons/watch.svg';

import './Node.css';

export namespace Node {
  export interface Props {
    id: Types.NodeId;
    nodeDetails: Types.NodeDetails;
    nodeStats: Types.NodeStats;
    blockDetails: Types.BlockDetails;
    location: Maybe<Types.NodeLocation>;
  }

  export interface PixelPosition {
    left: number;
    top: number;
  }

  export function Header() {
    return (
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
    )
  }

  export function Row(props: Props) {
    const [name, implementation, version] = props.nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = props.blockDetails;
    const [peers, txcount] = props.nodeStats;

    return (
      <tr>
        <td>{name}</td>
        <td style={{ width: 240 }}>{implementation} v{version}</td>
        <td style={{ width: 26 }}>{peers}</td>
        <td style={{ width: 26 }}>{txcount}</td>
        <td style={{ width: 88 }}>#{formatNumber(height)}</td>
        <td style={{ width: 154 }}><span title={hash}>{trimHash(hash, 16)}</span></td>
        <td style={{ width: 80 }}>{secondsWithPrecision(blockTime/1000)}</td>
        <td style={{ width: 58 }}>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td>
        <td style={{ width: 82 }}><Ago when={blockTimestamp} /></td>
      </tr>
    );
  }

  export function Location(props: Props & PixelPosition) {
    const { left, top } = props;
    const [name, implementation, version] = props.nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = props.blockDetails;

    return (
      <div className="Node-Location" style={{ left, top }}>
        <table className="Node-details">
          <tbody>
            <tr>
              <td><Icon src={nodeIcon} alt="Node" /></td><td colSpan={5}>{name}</td>
            </tr>
            <tr>
              <td><Icon src={nodeTypeIcon} alt="Implementation" /></td><td colSpan={5}>{implementation} v{version}</td>
            </tr>
            <tr>
              <td><Icon src={blockIcon} alt="Block" /></td><td colSpan={5}>#{formatNumber(height)}</td>
            </tr>
            <tr>
              <td><Icon src={blockHashIcon} alt="Block Hash" /></td><td colSpan={5}>{trimHash(hash, 20)}</td>
            </tr>
            <tr>
              <td><Icon src={blockTimeIcon} alt="Block Time" /></td>
              <td style={{ width: 80 }}>{secondsWithPrecision(blockTime/1000)}</td>
              <td><Icon src={propagationTimeIcon} alt="Block Propagation Time" /></td>
              <td style={{ width: 58 }}>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td>
              <td><Icon src={lastTimeIcon} alt="Last Block Time" /></td>
              <td style={{ minWidth: 82 }}><Ago when={blockTimestamp} /></td>
            </tr>
          </tbody>
        </table>
      </div>
    );
  }
}
