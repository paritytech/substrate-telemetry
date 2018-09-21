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
import cpuIcon from '../../icons/microchip-solid.svg';
import memoryIcon from '../../icons/memory-solid.svg';

import './Row.css';

interface RowProps {
  node: AppState.Node;
  settings: AppState.Settings;
};

interface HeaderProps {
  settings: AppState.Settings;
};

export default class Row extends React.Component<RowProps, {}> {
  public static Header = (props: HeaderProps) => {
    const { settings } = props;

    return (
      <thead>
        <tr>
          <th><Icon src={nodeIcon} alt="Node" /></th>
          { settings.validator ? <th style={{ width: 26 }}><Icon src={nodeValidatorIcon} alt="Validator" /></th> : null }
          { settings.implementation ? <th style={{ width: 240 }}><Icon src={nodeTypeIcon} alt="Implementation" /></th> : null }
          { settings.peers ? <th style={{ width: 26 }}><Icon src={peersIcon} alt="Peer Count" /></th> : null }
          { settings.txs ? <th style={{ width: 26 }}><Icon src={transactionsIcon} alt="Transactions in Queue" /></th> : null }
          { settings.cpu ? <th style={{ width: 26 }}><Icon src={cpuIcon} alt="% CPU use" /></th> : null }
          { settings.mem ? <th style={{ width: 26 }}><Icon src={memoryIcon} alt="Memory use" /></th> : null }
          { settings.blocknumber ? <th style={{ width: 88 }}><Icon src={blockIcon} alt="Block" /></th> : null }
          { settings.blockhash ? <th style={{ width: 154 }}><Icon src={blockHashIcon} alt="Block Hash" /></th> : null }
          { settings.blocktime ? <th style={{ width: 80 }}><Icon src={blockTimeIcon} alt="Block Time" /></th> : null }
          { settings.blockpropagation ? <th style={{ width: 58 }}><Icon src={propagationTimeIcon} alt="Block Propagation Time" /></th> : null }
          { settings.blocklasttime ? <th style={{ width: 100 }}><Icon src={lastTimeIcon} alt="Last Block Time" /></th> : null }
        </tr>
      </thead>
    )
  }

  public render() {
    const { nodeDetails, blockDetails, nodeStats } = this.props.node;
    const { settings } = this.props;

    const [name, implementation, version, validator] = nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = blockDetails;
    const [peers, txcount, memory, cpu] = nodeStats;
    const [semver] = version.match(SEMVER_PATTERN) || [version];

    let className = 'Node-Row';

    if (propagationTime != null) {
      className += ' Node-Row-synced';
    }

    return (
      <tr className={className}>
        <td>{name}</td>
        { settings.validator ? <td>{validator ? <span className="Node-Row-validator" title={validator}><Identicon id={validator} size={16} /></span> : '-'}</td> : null }
        { settings.implementation ? <td><span title={`${implementation} v${version}`}>{implementation} v{semver}</span></td> : null }
        { settings.peers ? <td>{peers}</td> : null }
        { settings.txs ? <td>{txcount}</td> : null }
        { settings.cpu ? <td>{cpu ? `${(cpu * 100).toFixed(1)}%` : '-'}</td> : null }
        { settings.mem ? <td>{memory ? <span title={`${memory}kb`}>{memory / 1024 | 0}mb</span> : '-'}</td> : null }
        { settings.blocknumber ? <td>#{formatNumber(height)}</td> : null }
        { settings.blockhash ? <td><span title={hash}>{trimHash(hash, 16)}</span></td> : null }
        { settings.blocktime ? <td>{secondsWithPrecision(blockTime/1000)}</td> : null }
        { settings.blockpropagation ? <td>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td> : null }
        { settings.blocklasttime ? <td><Ago when={blockTimestamp} /></td> : null }
      </tr>
    );
  }
}
