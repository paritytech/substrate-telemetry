import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../../utils';
import { Ago, Icon } from '../';
import { Props, PixelPosition } from './';
import { Types } from '@dotstats/common';

import nodeIcon from '../../icons/server.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import nodeLocationIcon from '../../icons/location.svg';
import blockIcon from '../../icons/package.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';

import './Location.css';

interface LocationState {
  hover: boolean;
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

    if (propagationTime != null) {
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
