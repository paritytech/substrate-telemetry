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
import { ColumnProps } from './';
import { Node } from '../../../state';
import { Tooltip, Icon } from '../../';
import icon from '../../../icons/terminal.svg';

import parityPolkadotIcon from '../../../icons/dot.svg';
import paritySubstrateIcon from '../../../icons/substrate.svg';
import polkadotJsIcon from '../../../icons/polkadot-js.svg';
import airalabRobonomicsIcon from '../../../icons/robonomics.svg';
import chainXIcon from '../../../icons/chainx.svg';
import edgewareIcon from '../../../icons/edgeware.svg';
import joystreamIcon from '../../../icons/joystream.svg';
import ladderIcon from '../../../icons/laddernetwork.svg';
import cennznetIcon from '../../../icons/cennznet.svg';
import crabIcon from '../../../icons/crab.svg';
import darwiniaIcon from '../../../icons/darwinia.svg';
import turingIcon from '../../../icons/turingnetwork.svg';
import dothereumIcon from '../../../icons/dothereum.svg';
import kagomeIcon from '../../../icons/kagome.svg';
import katalchainIcon from '../../../icons/katalchain.svg';
import bifrostIcon from '../../../icons/bifrost.svg';
import totemIcon from '../../../icons/totem.svg';
import nodleIcon from '../../../icons/nodle.svg';
import zeroIcon from '../../../icons/zero.svg';
import crustIcon from '../../../icons/crust.svg';
import gossamerIcon from '../../../icons/gossamer.svg';

const ICONS = {
  'parity-polkadot': parityPolkadotIcon,
  'Parity Polkadot': parityPolkadotIcon,
  'polkadot-js': polkadotJsIcon,
  'airalab-robonomics': airalabRobonomicsIcon,
  'substrate-node': paritySubstrateIcon,
  'Substrate Node': paritySubstrateIcon,
  'edgeware-node': edgewareIcon,
  'Edgeware Node': edgewareIcon,
  'joystream-node': joystreamIcon,
  ChainX: chainXIcon,
  'ladder-node': ladderIcon,
  'cennznet-node': cennznetIcon,
  'Darwinia Crab': crabIcon,
  Darwinia: darwiniaIcon,
  'turing-node': turingIcon,
  dothereum: dothereumIcon,
  'Kagome Node': kagomeIcon,
  katalchain: katalchainIcon,
  'bifrost-node': bifrostIcon,
  'totem-meccano-node': totemIcon,
  Totem: totemIcon,
  'Nodle Chain Node': nodleIcon,
  subzero: zeroIcon,
  Crust: crustIcon,
  Gossamer: gossamerIcon,
};
const SEMVER_PATTERN = /^\d+\.\d+\.\d+/;

export class ImplementationColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Implementation';
  public static readonly icon = icon;
  public static readonly width = 90;
  public static readonly setting = 'implementation';
  public static readonly sortBy = ({ sortableVersion }: Node) =>
    sortableVersion;

  private implementation: string;
  private version: string;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    if (this.props.node === nextProps.node) {
      // Implementation can't change unless we got a new node
      return false;
    }

    return (
      this.implementation !== nextProps.node.implementation ||
      this.version !== nextProps.node.version
    );
  }

  render() {
    const { implementation, version } = this.props.node;

    this.implementation = implementation;
    this.version = version;

    const [semver] = version.match(SEMVER_PATTERN) || ['?.?.?'];
    const implIcon = ICONS[implementation] || paritySubstrateIcon;

    return (
      <td className="Column">
        <Tooltip text={`${implementation} v${version}`} />
        <Icon src={implIcon} /> {semver}
      </td>
    );
  }
}
