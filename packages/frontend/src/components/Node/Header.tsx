import * as React from 'react';
import { Icon } from '../';

import nodeIcon from '../../icons/server.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import peersIcon from '../../icons/broadcast.svg';
import transactionsIcon from '../../icons/inbox.svg';
import blockIcon from '../../icons/package.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';

export function Header() {
  return (
    <thead>
      <tr>
        <th />
        <th><Icon src={nodeIcon} alt="Node" /></th>
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
