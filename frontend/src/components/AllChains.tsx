// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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
import { Types, Maybe } from '../common';
import { ChainData } from '../state';

import './AllChains.css';

export namespace AllChains {
  export interface Props {
    chains: ChainData[];
    subscribed: Maybe<Types.ChainLabel>;
    connection: Promise<Connection>;
  }
}

export class AllChains extends React.Component<AllChains.Props, {}> {
  public render() {
    const { chains, subscribed } = this.props;
    const close = subscribed ? `#list/${subscribed}` : '#list';

    return (
      <>
        <a className="AllChains-overlay" href={close} />
        <div className="AllChains">
          {chains.map((chain) => this.renderChain(chain))}
        </div>
      </>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;

    const className =
      label === this.props.subscribed
        ? 'AllChains-chain AllChains-chain-selected'
        : 'AllChains-chain';

    return (
      <a
        key={label}
        className={className}
        onClick={this.subscribe.bind(this, label)}
      >
        {label}{' '}
        <span className="AllChains-node-count" title="Node Count">
          {nodeCount}
        </span>
      </a>
    );
  }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
