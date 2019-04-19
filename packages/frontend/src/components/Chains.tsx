import * as React from 'react';
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '@dotstats/common';
import stable from 'stable';

import githubIcon from '../icons/mark-github.svg';
import './Chains.css';

interface ChainData {
  label: Types.ChainLabel;
  nodeCount: Types.NodeCount;
}

export namespace Chains {
  export interface Props {
    chains: Map<Types.ChainLabel, Types.NodeCount>,
    subscribed: Maybe<Types.ChainLabel>,
    connection: Promise<Connection>
  }
}

export class Chains extends React.Component<Chains.Props, {}> {
  public render() {
    return (
      <div className="Chains">
        <a className="Chains-fork-me" href="https://github.com/paritytech/substrate-telemetry" target="_blank">
          <Icon src={githubIcon} alt="Fork Me!" />
        </a>
        {
          this.chains.map((chain) => this.renderChain(chain))
        }
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
        <span className="Chains-node-label" title={label}>{label}</span>
        <span><span className="Chains-node-count" title="Node Count">{nodeCount}</span></span>
      </a>
    )
  }

  private get chains(): ChainData[] {
    return stable
      .inplace(
        Array.from(this.props.chains.entries()),
        (a, b) => b[1] - a[1]
      )
      .map(([label, nodeCount]) => ({ label, nodeCount }));
  }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
  }
}
