import * as React from 'react';
import { Connection } from '../Connection';
import { Types, Maybe } from '@dotstats/common';
import { ChainData } from '../state';

import './AllChains.css';

export namespace AllChains {
  export interface Props {
    chains: ChainData[],
    subscribed: Maybe<Types.ChainLabel>,
    connection: Promise<Connection>
  }
}

export class AllChains extends React.Component<AllChains.Props, {}> {
  public render() {
    const { chains, subscribed } = this.props;
    const close = subscribed ? `#list/${subscribed}` : '#list';

    return (
      <React.Fragment>
        <a className="AllChains-overlay" href={close} />
        <div className="AllChains">
          {chains.map((chain) => this.renderChain(chain))}
        </div>
      </React.Fragment>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;

    const className = label === this.props.subscribed
      ? 'AllChains-chain AllChains-chain-selected'
      : 'AllChains-chain';

    return (
      <a key={label} className={className} onClick={this.subscribe.bind(this, label)}>
        {label} <span className="AllChains-node-count" title="Node Count">{nodeCount}</span>
      </a>
    )
  }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
