// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import {
  formatNumber,
  trimHash,
  milliOrSecond,
  secondsWithPrecision,
} from '../../utils';
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

export type LocationQuarter = 0 | 1 | 2 | 3;

interface LocationProps {
  node: Node;
  position: LocationPosition;
  focused: boolean;
}

export interface LocationPosition {
  left: number;
  top: number;
  quarter: LocationQuarter;
}

interface LocationState {
  hover: boolean;
}

export class Location extends React.Component<LocationProps, LocationState> {
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
      <div
        className={className}
        style={{ left, top }}
        onMouseOver={this.onMouseOver}
        onMouseOut={this.onMouseOut}
      >
        {this.state.hover ? this.renderDetails() : null}
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

    let validatorRow = <div />;

    if (validator) {
      validatorRow = (
        <tr>
          <td>
            <Icon src={nodeValidatorIcon} />
          </td>
          <td colSpan={5}>
            {trimHash(validator, 30)}
            <span className="Location-validator">
              <PolkadotIcon account={validator} size={16} />
            </span>
          </td>
        </tr>
      );
    }

    return (
      <table className="Location-details Location-details">
        <tbody>
          <tr>
            <td title="Node">
              <Icon src={nodeIcon} />
            </td>
            <td colSpan={5}>{name}</td>
          </tr>
          {validatorRow}
          <tr>
            <td title="Implementation">
              <Icon src={nodeTypeIcon} />
            </td>
            <td colSpan={5}>
              {implementation} v{version}
            </td>
          </tr>
          <tr>
            <td title="Location">
              <Icon src={nodeLocationIcon} />
            </td>
            <td colSpan={5}>{city}</td>
          </tr>
          <tr>
            <td title="Block">
              <Icon src={blockIcon} />
            </td>
            <td colSpan={5}>#{formatNumber(height)}</td>
          </tr>
          <tr>
            <td title="Block Hash">
              <Icon src={blockHashIcon} />
            </td>
            <td colSpan={5}>{trimHash(hash, 20)}</td>
          </tr>
          <tr>
            <td title="Block Time">
              <Icon src={blockTimeIcon} />
            </td>
            <td style={{ width: 80 }}>
              {secondsWithPrecision(blockTime / 1000)}
            </td>
            <td title="Block Propagation Time">
              <Icon src={propagationTimeIcon} />
            </td>
            <td style={{ width: 58 }}>
              {propagationTime == null ? '∞' : milliOrSecond(propagationTime)}
            </td>
            <td title="Last Block Time">
              <Icon src={lastTimeIcon} />
            </td>
            <td style={{ minWidth: 82 }}>
              <Ago when={blockTimestamp} />
            </td>
          </tr>
        </tbody>
      </table>
    );
  }

  private onMouseOver = () => {
    this.setState({ hover: true });
  };

  private onMouseOut = () => {
    this.setState({ hover: false });
  };
}
