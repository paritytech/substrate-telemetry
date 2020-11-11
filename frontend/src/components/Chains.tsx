import * as React from 'react';
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '../common';
import { ChainData } from '../state';

import githubIcon from '../icons/mark-github.svg';
import listIcon from '../icons/kebab-horizontal.svg';
import './Chains.css';

export namespace Chains {
  export interface Props {
    chains: ChainData[];
    subscribed: Maybe<Types.ChainLabel>;
    connection: Promise<Connection>;
  }
}

// How many chains should be rendered in the DOM
const VISIBLE_CAP = 16;
// Milliseconds, sets the minimum time between the renders
const RENDER_THROTTLE = 1000;

export class Chains extends React.Component<Chains.Props, {}> {
  private lastRender = performance.now();
  private clicked: Maybe<Types.ChainLabel>;

  public shouldComponentUpdate(nextProps: Chains.Props) {
    if (nextProps.subscribed !== this.clicked) {
      this.clicked = nextProps.subscribed;
    }

    return (
      this.props.subscribed !== nextProps.subscribed ||
      performance.now() - this.lastRender > RENDER_THROTTLE
    );
  }

  public render() {
    this.lastRender = performance.now();

    const allChainsHref = this.props.subscribed
      ? `#all-chains/${this.props.subscribed}`
      : `#all-chains`;
    const { chains } = this.props;

    return (
      <div className="Chains">
        {chains.slice(0, VISIBLE_CAP).map((chain) => this.renderChain(chain))}
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
        >
          <Icon src={githubIcon} />
        </a>
      </div>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;

    let className = 'Chains-chain';

    if (label === this.props.subscribed) {
      className += ' Chains-chain-selected';
    }

    return (
      <a
        key={label}
        className={className}
        onClick={this.subscribe.bind(this, label)}
      >
        {label}
        <span className="Chains-node-count" title="Node Count">
          {nodeCount}
        </span>
      </a>
    );
  }

  private async subscribe(chain: Types.ChainLabel) {
    if (chain === this.clicked) {
      return;
    }
    this.clicked = chain;

    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
