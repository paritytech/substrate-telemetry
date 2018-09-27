import * as React from 'react';
import Identicon from 'polkadot-identicon';
import { Types } from '@dotstats/common';
import { formatNumber, milliOrSecond, secondsWithPrecision } from '../../utils';
import { State as AppState } from '../../state';
import { PersistentSet } from '../../persist';
import { SEMVER_PATTERN } from './';
import { Ago, Icon } from '../';

import nodeIcon from '../../icons/server.svg';
import nodeLocationIcon from '../../icons/location.svg';
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

import parityPolkadotIcon from '../../icons/dot.svg';
import paritySubstrateIcon from '../../icons/parity-substrate.svg';
import unknownImplementationIcon from '../../icons/question-solid.svg';

import './Row.css';

interface RowProps {
  node: AppState.Node;
  settings: AppState.Settings;
  pins: PersistentSet<Types.NodeName>;
};

interface HeaderProps {
  settings: AppState.Settings;
};

interface Column {
  label: string;
  icon: string;
  width?: number;
  setting?: keyof AppState.Settings;
  render: (node: AppState.Node) => React.ReactElement<any> | string;
}

function Truncate(props: { text: string }): React.ReactElement<any> {
  const { text } = props;

  return <div className="Node-Row-truncate" title={text}>{text}</div>;
}

export default class Row extends React.Component<RowProps, {}> {
  public static readonly columns: Column[] = [
    {
      label: 'Node',
      icon: nodeIcon,
      render: ({ nodeDetails }) => <Truncate text={nodeDetails[0]} />
    },
    {
      label: 'Validator',
      icon: nodeValidatorIcon,
      width: 26,
      setting: 'validator',
      render: ({ nodeDetails }) => {
        const validator = nodeDetails[3];

        return validator ? <span className="Node-Row-validator" title={validator}><Identicon id={validator} size={16} /></span> : '-';
      }
    },
    {
      label: 'Location',
      icon: nodeLocationIcon,
      width: 140,
      setting: 'location',
      render: ({ location }) => location && location[2] ? <Truncate text={location[2]} /> : '-'
    },
    {
      label: 'Implementation',
      icon: nodeTypeIcon,
      width: 90,
      setting: 'implementation',
      render: ({ nodeDetails }) => {
        const [, implementation, version] = nodeDetails;
        const [semver] = version.match(SEMVER_PATTERN) || [version];
        const implIcon = implementation === 'parity-polkadot' ? parityPolkadotIcon
                       : implementation === 'substrate-node' ? paritySubstrateIcon
                       : unknownImplementationIcon;

        return <span title={`${implementation} v${version}`}><Icon src={implIcon} /> {semver}</span>;
      }
    },
    {
      label: 'Peer Count',
      icon: peersIcon,
      width: 26,
      setting: 'peers',
      render: ({ nodeStats }) => `${nodeStats[0]}`
    },
    {
      label: 'Transactions in Queue',
      icon: transactionsIcon,
      width: 26,
      setting: 'txs',
      render: ({ nodeStats }) => `${nodeStats[1]}`
    },
    {
      label: '% CPU Use',
      icon: cpuIcon,
      width: 26,
      setting: 'cpu',
      render: ({ nodeStats }) => {
        const cpu = nodeStats[3];

        return cpu ? `${cpu.toFixed(1)}%` : '-';
      }
    },
    {
      label: 'Memory Use',
      icon: memoryIcon,
      width: 26,
      setting: 'mem',
      render: ({ nodeStats }) => {
        const memory = nodeStats[2];

        return memory ? <span title={`${memory}kb`}>{memory / 1024 | 0}mb</span> : '-';
      }
    },
    {
      label: 'Block',
      icon: blockIcon,
      width: 88,
      setting: 'blocknumber',
      render: ({ blockDetails }) => `#${formatNumber(blockDetails[0])}`
    },
    {
      label: 'Block Hash',
      icon: blockHashIcon,
      width: 154,
      setting: 'blockhash',
      render: ({ blockDetails }) => <Truncate text={blockDetails[1]} />
    },
    {
      label: 'Block Time',
      icon: blockTimeIcon,
      width: 80,
      setting: 'blocktime',
      render: ({ blockDetails }) => `${secondsWithPrecision(blockDetails[2]/1000)}`
    },
    {
      label: 'Block Propagation Time',
      icon: propagationTimeIcon,
      width: 58,
      setting: 'blockpropagation',
      render: ({ blockDetails }) => {
        const propagationTime = blockDetails[4];

        return propagationTime === null ? '∞' : milliOrSecond(propagationTime as number);
      }
    },
    {
      label: 'Last Block Time',
      icon: lastTimeIcon,
      width: 100,
      setting: 'blocklasttime',
      render: ({ blockDetails }) => <Ago when={blockDetails[3]} />
    },
  ];

  public static Header = (props: HeaderProps) => {
    const { settings } = props;

    return (
      <thead>
        <tr className="Node-Row-Header">
          {
            Row.columns
              .filter(({ setting }) => setting == null || settings[setting])
              .map(({ icon, width, label }, index) => (
                <th key={index} style={width ? { width } : undefined}><Icon src={icon} alt={label} /></th>
              ))
          }
        </tr>
      </thead>
    )
  }

  public render() {
    const { node, settings } = this.props;
    const propagationTime = node.blockDetails[4];

    let className = 'Node-Row';

    if (propagationTime != null) {
      className += ' Node-Row-synced';
    }

    if (node.pinned) {
      className += ' Node-Row-pinned';
    }

    return (
      <tr className={className} onClick={this.toggle}>
        {
          Row.columns
            .filter(({ setting }) => setting == null || settings[setting])
            .map(({ render }, index) => <td key={index}>{render(node)}</td>)
        }
      </tr>
    );
  }

  public toggle = () => {
    const { pins, node } = this.props;
    const name = node.nodeDetails[0];

    if (node.pinned) {
      pins.delete(name)
    } else {
      pins.add(name);
    }
  }
}
