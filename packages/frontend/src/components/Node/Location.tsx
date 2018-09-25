import * as React from 'react';
import Identicon from 'polkadot-identicon';
import { Types } from '@dotstats/common';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../../utils';
import { Ago, Icon } from '../';
import { State as AppState } from '../../state';

import nodeIcon from '../../icons/server.svg';
import nodeValidatorIcon from '../../icons/shield.svg';
import nodeTypeIcon from '../../icons/terminal.svg';
import nodeLocationIcon from '../../icons/location.svg';
import blockIcon from '../../icons/package.svg';
import blockHashIcon from '../../icons/file-binary.svg';
import blockTimeIcon from '../../icons/history.svg';
import propagationTimeIcon from '../../icons/dashboard.svg';
import lastTimeIcon from '../../icons/watch.svg';

import './Location.css';

namespace Location {
  export type Quarter = 0 | 1 | 2 | 3;

  export interface Props {
    node: AppState.Node;
    position: Position;
    focused: boolean;
  }

  export interface Position {
    left: number;
    top: number;
    quarter: Quarter;
  }

  export interface State {
    hover: boolean;
  }
}

class Location extends React.Component<Location.Props, Location.State> {
  public readonly state = { hover: false };

  public render() {
    const { node, position, focused } = this.props;
    const { left, top, quarter } = position;
    const { blockDetails, location } = node;
    const height = blockDetails[0];
    const propagationTime = blockDetails[4];

    if (!location) {
      return null;
    }

    let className = `Node-Location Node-Location-quarter${quarter}`;

    if (focused) {
      if (propagationTime != null) {
        className += ' Node-Location-synced';
      } else if (height % 2 === 1) {
        className += ' Node-Location-odd';
      }
    } else {
      className += ' Node-Location-dimmed';
    }

    return (
      <div className={className} style={{ left, top }} onMouseOver={this.onMouseOver} onMouseOut={this.onMouseOut}>
      {
        this.state.hover ? this.renderDetails(location) : null
      }
        <div className="Node-Location-ping" />
      </div>
    );
  }

  private renderDetails(location: Types.NodeLocation) {
    const { node } = this.props;
    const [name, implementation, version, validator] = node.nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = node.blockDetails;

    let validatorRow = null;

    if (validator) {
      validatorRow = (
        <tr>
          <td><Icon src={nodeValidatorIcon} alt="Node" /></td>
          <td colSpan={5}>
            {trimHash(validator, 30)}
            <span className="Node-Location-validator"><Identicon id={validator} size={16} /></span>
          </td>
        </tr>
      );
    }

    return (
      <table className="Node-Location-details Node-Location-details">
        <tbody>
          <tr>
            <td><Icon src={nodeIcon} alt="Node" /></td><td colSpan={5}>{name}</td>
          </tr>
          {validatorRow}
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

export default Location;
