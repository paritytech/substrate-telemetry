import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../utils';
import { Ago, Icon } from './';
import { Types, Maybe } from '@dotstats/common';

import nodeIcon from '../icons/server.svg';
import nodeTypeIcon from '../icons/terminal.svg';
import nodeLocationIcon from '../icons/location.svg';
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

  export interface LocationState {
    hover: boolean;
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

  export class Location extends React.Component<Props & PixelPosition, LocationState> {
    public readonly state = { hover: false };

    public render() {
      const { left, top, location } = this.props;
      const height = this.props.blockDetails[0];
      const propagationTime = this.props.blockDetails[4];

      if (!location) {
        return null;
      }

      let className = 'Node-Location';

      if (propagationTime) {
        className += ' Node-Location-synced';
      } else if (height % 2 === 1) {
        className += ' Node-Location-odd';
      }

      return (
        <div className={className} style={{ left, top }} onMouseOver={this.onMouseOver} onMouseOut={this.onMouseOut}>
        {
          this.state.hover ? this.renderDetails(location) : null
        }
        </div>
      );
    }

    private renderDetails(location: Types.NodeLocation) {
      const [name, implementation, version] = this.props.nodeDetails;
      const [height, hash, blockTime, blockTimestamp, propagationTime] = this.props.blockDetails;

      return (
        <table className="Node-details">
          <tbody>
            <tr>
              <td><Icon src={nodeIcon} alt="Node" /></td><td colSpan={5}>{name}</td>
            </tr>
            <tr>
              <td><Icon src={nodeTypeIcon} alt="Implementation" /></td><td colSpan={5}>{implementation} v{version}</td>
            </tr>
            <tr>
              <td><Icon src={nodeLocationIcon} alt="Location" /></td><td colSpan={5}>{location[2]}</td>
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
      );
    }

    private onMouseOver = () => {
      this.setState({ hover: true });
    }

    private onMouseOut = () => {
      this.setState({ hover: false });
    }
  }
}
