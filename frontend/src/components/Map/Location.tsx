import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../../utils';
import { Ago, Icon, PolkadotIcon } from '../';
import { Node } from '../../state';

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

export namespace Location {
  export type Quarter = 0 | 1 | 2 | 3;

  export interface Props {
    node: Node;
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

export class Location extends React.Component<Location.Props, Location.State> {
  public readonly state = { hover: false };

  public render() {
    const { node, position, focused } = this.props;
    const { left, top, quarter } = position;
    const { height, propagationTime, city } = node;

    if (!city) {
      return null;
    }

    let className = `Location Location-quarter${quarter}`;

    if (focused) {
      if (propagationTime != null) {
        className += ' Location-synced';
      } else if (height % 2 === 1) {
        className += ' Location-odd';
      }
    } else {
      className += ' Location-dimmed';
    }

    return (
      <div className={className} style={{ left, top }} onMouseOver={this.onMouseOver} onMouseOut={this.onMouseOut}>
      {
        this.state.hover ? this.renderDetails() : null
      }
        <div className="Location-ping" />
      </div>
    );
  }

  private renderDetails() {
    const {
      name,
      implementation,
      version,
      validator,
      height,
      hash,
      blockTime,
      blockTimestamp,
      propagationTime,
      city,
    } = this.props.node;

    let validatorRow = null;

    if (validator) {
      validatorRow = (
        <tr>
          <td><Icon src={nodeValidatorIcon} alt="Node" /></td>
          <td colSpan={5}>
            {trimHash(validator, 30)}
            <span className="Location-validator"><PolkadotIcon account={validator} size={16} /></span>
          </td>
        </tr>
      );
    }

    return (
      <table className="Location-details Location-details">
        <tbody>
          <tr>
            <td><Icon src={nodeIcon} alt="Node" /></td><td colSpan={5}>{name}</td>
          </tr>
          {validatorRow}
          <tr>
            <td><Icon src={nodeTypeIcon} alt="Implementation" /></td><td colSpan={5}>{implementation} v{version}</td>
          </tr>
          <tr>
            <td><Icon src={nodeLocationIcon} alt="Location" /></td><td colSpan={5}>{city}</td>
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
            <td style={{ width: 58 }}>{propagationTime == null ? '∞' : milliOrSecond(propagationTime)}</td>
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
