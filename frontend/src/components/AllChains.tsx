import * as React from 'react';
import { Connection } from '../Connection';
import { Types, Maybe } from '../common';
import { ChainData } from '../state';

import './AllChains.css';

export namespace AllChains {
  export interface Props {
    chains: ChainData[];
    subscribed: Maybe<Types.GenesisHash>;
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
    const { label, genesisHash, nodeCount } = chain;

    const className =
      genesisHash === this.props.subscribed
        ? 'AllChains-chain AllChains-chain-selected'
        : 'AllChains-chain';

    return (
      <a
        key={label}
        className={className}
        onClick={this.subscribe.bind(this, genesisHash)}
      >
        {label}{' '}
        <span className="AllChains-node-count" title="Node Count">
          {nodeCount}
        </span>
      </a>
    );
  }

  private async subscribe(chain: Types.GenesisHash) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
