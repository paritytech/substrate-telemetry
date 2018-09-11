import * as React from 'react';
import Identicon from 'polkadot-identicon';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../../utils';
import { State as AppState } from '../../state';
import { SEMVER_PATTERN } from './';
import { Ago, Icon } from '../';

import nodeIcon from '../../icons/server.svg';
import nodeValidatorIcon from '../../icons/shield.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import peersIcon from '../../icons/broadcast.svg';
import transactionsIcon from '../../icons/inbox.svg';
import blockIcon from '../../icons/package.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';

import './Row.css';

export default class Row extends React.Component<AppState.Node, {}> {
  public static Header = () => {
    return (
      <thead>
        <tr>
          <th><Icon src={nodeIcon} alt="Node" /></th>
          <th style={{ width: 26 }}><Icon src={nodeValidatorIcon} alt="Validator" /></th>
          <th style={{ width: 240 }}><Icon src={nodeTypeIcon} alt="Implementation" /></th>
          <th style={{ width: 26 }}><Icon src={peersIcon} alt="Peer Count" /></th>
          <th style={{ width: 26 }}><Icon src={transactionsIcon} alt="Transactions in Queue" /></th>
          <th style={{ width: 88 }}><Icon src={blockIcon} alt="Block" /></th>
          <th style={{ width: 154 }}><Icon src={blockHashIcon} alt="Block Hash" /></th>
          <th style={{ width: 80 }}><Icon src={blockTimeIcon} alt="Block Time" /></th>
          <th style={{ width: 58 }}><Icon src={propagationTimeIcon} alt="Block Propagation Time" /></th>
          <th style={{ width: 100 }}><Icon src={lastTimeIcon} alt="Last Block Time" /></th>
        </tr>
      </thead>
    )
  }

  public render() {
    const { nodeDetails, blockDetails, nodeStats } = this.props;

    const [name, implementation, version, validator] = nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = blockDetails;
    const [peers, txcount] = nodeStats;
    const [semver] = version.match(SEMVER_PATTERN) || [version];

    let className = 'Node-Row';

    if (propagationTime != null) {
      className += ' Node-Row-synced';
    }

    return (
      <tr className={className}>
        <td>{name}</td>
        <td>{validator ? <span title={validator}><Identicon id={validator} size={16} /></span> : null}</td>
        <td><span title={`${implementation} v${version}`}>{implementation} v{semver}</span></td>
        <td>{peers}</td>
        <td>{txcount}</td>
        <td>#{formatNumber(height)}</td>
        <td><span title={hash}>{trimHash(hash, 16)}</span></td>
        <td>{secondsWithPrecision(blockTime/1000)}</td>
        <td>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td>
        <td><Ago when={blockTimestamp} /></td>
      </tr>
    );
  }
}
