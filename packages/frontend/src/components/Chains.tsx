import * as React from 'react';
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '@dotstats/common';
import { ChainData } from '../state';

import githubIcon from '../icons/mark-github.svg';
import listIcon from '../icons/three-bars.svg';
import './Chains.css';

export namespace Chains {
  export interface Props {
    chains: ChainData[],
    subscribed: Maybe<Types.ChainLabel>,
    connection: Promise<Connection>
  }
}

export class Chains extends React.Component<Chains.Props, {}> {
  public render() {
    const allChainsHref = this.props.subscribed ? `#all-chains/${this.props.subscribed}` : `#all-chains`;
    const { chains } = this.props;

    return (
      <div className="Chains">
        {chains.map((chain) => this.renderChain(chain))}
        <a className="Chains-all-chains" href={allChainsHref}>
          <Icon src={listIcon} alt="All Chains" />
        </a>
        <a className="Chains-fork-me" href="https://github.com/paritytech/substrate-telemetry" target="_blank">
          <Icon src={githubIcon} alt="Fork Me!" />
        </a>
      </div>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;

    const className = label === this.props.subscribed
      ? 'Chains-chain Chains-chain-selected'
      : 'Chains-chain';

    return (
      <a key={label} className={className} onClick={this.subscribe.bind(this, label)}>
        {label} <span className="Chains-node-count" title="Node Count">{nodeCount}</span>
      </a>
    )
  }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
