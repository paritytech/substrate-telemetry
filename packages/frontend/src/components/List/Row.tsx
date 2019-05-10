import * as React from 'react';
import Identicon from 'polkadot-identicon';
import { Types, Maybe, timestamp } from '@dotstats/common';
import { formatNumber, getHashData, milliOrSecond, secondsWithPrecision } from '../../utils';
import { State as AppState, Node } from '../../state';
import { PersistentSet } from '../../persist';
import { Truncate } from './';
import { Ago, Icon, Tooltip, Sparkline } from '../';

import nodeIcon from '../../icons/server.svg';
import nodeLocationIcon from '../../icons/location.svg';
import nodeValidatorIcon from '../../icons/shield.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import networkIdIcon from '../../icons/fingerprint.svg';
import peersIcon from '../../icons/broadcast.svg';
import transactionsIcon from '../../icons/inbox.svg';
import blockIcon from '../../icons/cube.svg';
import finalizedIcon from '../../icons/cube-alt.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';
import cpuIcon from '../../icons/microchip-solid.svg';
import memoryIcon from '../../icons/memory-solid.svg';
import uploadIcon from '../../icons/cloud-upload.svg';
import downloadIcon from '../../icons/cloud-download.svg';
import networkIcon from '../../icons/network.svg';
import externalLinkIcon from '../../icons/link-external.svg';

import parityPolkadotIcon from '../../icons/dot.svg';
import paritySubstrateIcon from '../../icons/substrate.svg';
import polkadotJsIcon from '../../icons/polkadot-js.svg';
import airalabRobonomicsIcon from '../../icons/robonomics.svg';
import chainXIcon from '../../icons/chainx.svg';
import edgewareIcon from '../../icons/edgeware.svg';
import joystreamIcon from '../../icons/joystream.svg';
import ladderIcon from '../../icons/laddernetwork.svg';
import unknownImplementationIcon from '../../icons/question-solid.svg';

import './Row.css';

const SEMVER_PATTERN = /^\d+\.\d+\.\d+/;
const BANDWIDTH_SCALE = 1024 * 1024;
const MEMORY_SCALE = 2 * 1024 * 1024;
const ICONS = {
  'parity-polkadot': parityPolkadotIcon,
  'polkadot-js': polkadotJsIcon,
  'robonomics-node': airalabRobonomicsIcon,
  'substrate-node': paritySubstrateIcon,
  'edgeware-node': edgewareIcon,
  'joystream-node': joystreamIcon,
  'ChainX': chainXIcon,
  'ladder-node': ladderIcon,
};

export namespace Row {
  export interface Props {
    node: Node;
    pins: PersistentSet<Types.NodeName>;
    columns: Column[];
  }

  export interface State {
    update: number;
  }
}

interface HeaderProps {
  columns: Column[];
}

interface Column {
  label: string;
  icon: string;
  width?: number;
  setting?: keyof AppState.Settings;
  render: (node: Node) => React.ReactElement<any> | string;
}

function formatStamp(stamp: Types.Timestamp): string {
  const passed = (timestamp() - stamp) / 1000 | 0;

  const hours = passed / 3600 | 0;
  const minutes = (passed % 3600) / 60 | 0;
  const seconds = (passed % 60) | 0;

  return hours ? `${hours}h ago`
       : minutes ? `${minutes}m ago`
       : `${seconds}s ago`;
}

function formatMemory(kbs: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';
  const mbs = kbs / 1024 | 0;

  if (mbs >= 1000) {
    return `${(mbs / 1024).toFixed(1)} GB${ago}`;
  } else {
    return `${mbs} MB${ago}`;
  }
}

function formatBandwidth(bps: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';

  if (bps >= 1024 * 1024) {
    return `${(bps / (1024 * 1024)).toFixed(1)} MB/s${ago}`;
  } else if (bps >= 1000) {
    return `${(bps / 1024).toFixed(1)} kB/s${ago}`;
  } else {
    return `${bps | 0} B/s${ago}`;
  }
}

function formatCPU(cpu: number, stamp: Maybe<Types.Timestamp>): string {
  const ago = stamp ? ` (${formatStamp(stamp)})` : '';
  const fractionDigits = cpu > 100 ? 0
                       : cpu > 10 ? 1
                       : cpu > 1 ? 2
                       : 3;

  return `${cpu.toFixed(fractionDigits)}%${ago}`;
}

const URI_BASE = window.location.protocol === 'https:'
                                    ? `/network_state/`
                                    : `http://${window.location.hostname}:8081/network_state/`;

export class Row extends React.Component<Row.Props, Row.State> {
  public static readonly columns: Column[] = [
    {
      label: 'Node',
      icon: nodeIcon,
      render: ({ name }) => <Truncate text={name} position="left" />
    },
    {
      label: 'Validator',
      icon: nodeValidatorIcon,
      width: 16,
      setting: 'validator',
      render: ({ validator }) => {
        return validator ? <Tooltip text={validator} copy={true}><span className="Row-validator"><Identicon account={validator} size={16} /></span></Tooltip> : '-';
      }
    },
    {
      label: 'Location',
      icon: nodeLocationIcon,
      width: 140,
      setting: 'location',
      render: ({ city }) => city ? <Truncate position="left" text={city} /> : '-'
    },
    {
      label: 'Implementation',
      icon: nodeTypeIcon,
      width: 90,
      setting: 'implementation',
      render: ({ implementation, version }) => {
        const [semver] = version.match(SEMVER_PATTERN) || ['?.?.?'];
        const implIcon = ICONS[implementation] || unknownImplementationIcon;

        return (
          <Tooltip text={`${implementation} v${version}`}>
            <Icon src={implIcon} /> {semver}
          </Tooltip>
        );
      }
    },
    {
      label: 'Network ID',
      icon: networkIdIcon,
      width: 90,
      setting: 'networkId',
      render: ({ networkId }) => networkId ? <Truncate position="left" text={networkId} /> : '-'
    },
    {
      label: 'Peer Count',
      icon: peersIcon,
      width: 26,
      setting: 'peers',
      render: ({ peers }) => `${peers}`
    },
    {
      label: 'Transactions in Queue',
      icon: transactionsIcon,
      width: 26,
      setting: 'txs',
      render: ({ txs }) => `${txs}`
    },
    {
      label: '% CPU Use',
      icon: cpuIcon,
      width: 40,
      setting: 'cpu',
      render: ({ cpu, chartstamps }) => {
        if (cpu.length < 3) {
          return '-';
        }

        return (
          <Sparkline width={44} height={16} stroke={1} format={formatCPU} values={cpu} stamps={chartstamps} minScale={100} />
        );
      }
    },
    {
      label: 'Memory Use',
      icon: memoryIcon,
      width: 40,
      setting: 'mem',
      render: ({ mem, chartstamps }) => {
        if (mem.length < 3) {
          return '-';
        }

        return (
          <Sparkline width={44} height={16} stroke={1} format={formatMemory} values={mem} stamps={chartstamps} minScale={MEMORY_SCALE} />
        );
      }
    },
    {
      label: 'Upload Bandwidth',
      icon: uploadIcon,
      width: 40,
      setting: 'upload',
      render: ({ upload, chartstamps }) => {
        if (upload.length < 3) {
          return '-';
        }

        return (
          <Sparkline width={44} height={16} stroke={1} format={formatBandwidth} values={upload} stamps={chartstamps} minScale={BANDWIDTH_SCALE} />
        );
      }
    },
    {
      label: 'Download Bandwidth',
      icon: downloadIcon,
      width: 40,
      setting: 'download',
      render: ({ download, chartstamps }) => {
        if (download.length < 3) {
          return '-';
        }

        return (
          <Sparkline width={44} height={16} stroke={1} format={formatBandwidth} values={download} stamps={chartstamps} minScale={BANDWIDTH_SCALE} />
        );
      }
    },
    {
      label: 'Block',
      icon: blockIcon,
      width: 88,
      setting: 'blocknumber',
      render: ({ height }) => `#${formatNumber(height)}`
    },
    {
      label: 'Block Hash',
      icon: blockHashIcon,
      width: 154,
      setting: 'blockhash',
      render: ({ hash }) => <Truncate position="right" text={hash} copy={true} />
    },
    {
      label: 'Finalized Block',
      icon: finalizedIcon,
      width: 88,
      setting: 'finalized',
      render: ({ finalized }) => `#${formatNumber(finalized)}`
    },
    {
      label: 'Finalized Block Hash',
      icon: blockHashIcon,
      width: 154,
      setting: 'finalizedhash',
      render: ({ finalizedHash }) => <Truncate position="right" text={finalizedHash} copy={true} />
    },
    {
      label: 'Block Time',
      icon: blockTimeIcon,
      width: 80,
      setting: 'blocktime',
      render: ({ blockTime }) => `${secondsWithPrecision(blockTime/1000)}`
    },
    {
      label: 'Block Propagation Time',
      icon: propagationTimeIcon,
      width: 58,
      setting: 'blockpropagation',
      render: ({ propagationTime }) => propagationTime == null ? '∞' : milliOrSecond(propagationTime)
    },
    {
      label: 'Last Block Time',
      icon: lastTimeIcon,
      width: 100,
      setting: 'blocklasttime',
      render: ({ blockTimestamp }) => <Ago when={blockTimestamp} />
    },
    {
      label: 'NetworkState',
      icon: networkIcon,
      width: 16,
      setting: 'networkstate',
      render: ({ id }) => {
        const chainLabel = getHashData().chain;

        if (!chainLabel) {
          return '-';
        }

        const uri = `${URI_BASE}${encodeURIComponent(chainLabel)}/${id}/`;
        return <a href={uri} target="_blank"><Icon src={externalLinkIcon} /></a>;
      },
    },
  ];

  public static Header = (props: HeaderProps) => {
    const { columns } = props;
    const last = columns.length - 1;

    return (
      <thead>
        <tr className="Row-Header">
          {
            columns.map(({ icon, width, label }, index) => {
              const position = index === 0 ? 'left'
                             : index === last ? 'right'
                             : 'center';

              return (
                <th key={index} style={width ? { width } : undefined}>
                  <Tooltip text={label} inline={true} position={position}><Icon src={icon} /></Tooltip>
                </th>
              )
            })
          }
        </tr>
      </thead>
    )
  }

  public state = { update: 0 };

  public componentDidMount() {
    const { node } = this.props;

    node.subscribe(this.onUpdate);
  }

  public componentWillUnmount() {
    const { node } = this.props;

    node.unsubscribe(this.onUpdate);
  }

  public shouldComponentUpdate(nextProps: Row.Props, nextState: Row.State): boolean {
    return this.props.node.id !== nextProps.node.id || this.state.update !== nextState.update;
  }

  public render() {
    const { node, columns } = this.props;

    let className = 'Row';

    if (node.propagationTime != null) {
      className += ' Row-synced';
    }

    if (node.pinned) {
      className += ' Row-pinned';
    }

    return (
      <tr className={className} onClick={this.toggle}>
        {
          columns.map(({ render }, index) => <td key={index}>{render(node)}</td>)
        }
      </tr>
    );
  }

  public toggle = () => {
    const { pins, node } = this.props;

    if (node.pinned) {
      pins.delete(node.name)
    } else {
      pins.add(node.name);
    }
  }

  private onUpdate = () => {
    this.setState({ update: this.state.update + 1 });
  }
}
