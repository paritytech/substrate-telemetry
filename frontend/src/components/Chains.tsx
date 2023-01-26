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
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '../common';
import { ChainData } from '../state';
import { viewport } from '../utils';

import githubIcon from '../icons/mark-github.svg';
import listIcon from '../icons/kebab-horizontal.svg';
import './Chains.css';

interface ChainsProps {
  chains: ChainData[];
  subscribedHash: Maybe<Types.GenesisHash>;
  subscribedData: Maybe<ChainData>;
  connection: Promise<Connection>;
}

// Average tab width in pixels
const AVERAGE_TAB_WIDTH = 160;
// Milliseconds, sets the minimum time between the renders
const RENDER_THROTTLE = 1000;

export class Chains extends React.Component<ChainsProps> {
  private lastRender = performance.now();
  private clicked: Maybe<Types.GenesisHash>;
  private subscribedChainInView = false;

  public shouldComponentUpdate(nextProps: ChainsProps) {
    if (nextProps.subscribedHash !== this.clicked) {
      this.clicked = nextProps.subscribedHash;
    }

    return (
      this.props.subscribedHash !== nextProps.subscribedHash ||
      performance.now() - this.lastRender > RENDER_THROTTLE
    );
  }

  public render() {
    this.lastRender = performance.now();
    this.subscribedChainInView = false;

    const viewportWidth = viewport().width;
    const { chains, subscribedHash, subscribedData } = this.props;

    const renderedChains = chains
      .slice(0, (viewportWidth / AVERAGE_TAB_WIDTH) | 0)
      .map((data) => this.renderChain(data));

    const allChainsHref = subscribedHash
      ? `#all-chains/${subscribedHash}`
      : '#all-chains';

    const subscribedChain =
      subscribedData && !this.subscribedChainInView ? (
        <div className="Chains-extra-subscribed-chain">
          {this.renderChain(subscribedData)}
        </div>
      ) : null;

    return (
      <div className="Chains">
        {subscribedChain}
        {renderedChains}
        <a
          className="Chains-all-chains"
          href={allChainsHref}
          title="All Chains"
        >
          <Icon src={listIcon} />
        </a>
        <a
          className="Chains-fork-me"
          href="https://github.com/paritytech/substrate-telemetry"
          target="_blank"
          title="Fork Me!"
          rel="noreferrer"
        >
          <Icon src={githubIcon} />
        </a>
      </div>
    );
  }

  private renderChain(chainData: ChainData): React.ReactNode {
    const { label, genesisHash, nodeCount } = chainData;

    let className = 'Chains-chain';

    if (genesisHash === this.props.subscribedHash) {
      className += ' Chains-chain-selected';
      this.subscribedChainInView = true;
    }

    return (
      <a
        key={genesisHash}
        className={className}
        onClick={this.subscribe.bind(this, genesisHash)}
      >
        <span>{label}</span>
        <span className="Chains-node-count" title="Node Count">
          {nodeCount}
        </span>
      </a>
    );
  }

  private async subscribe(chain: Types.GenesisHash) {
    if (chain === this.clicked) {
      return;
    }
    this.clicked = chain;

    const connection = await this.props.connection;

    connection.subscribe(chain);
  }
}
