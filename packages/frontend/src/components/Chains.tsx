import * as React from 'react';
import { Connection } from '../message';
import { Icon } from './Icon';
import { Types, Maybe } from '@dotstats/common';

import chainIcon from '../icons/link.svg';
import './Chains.css';

export namespace Chains {
    export interface Props {
        chains: Set<Types.ChainLabel>,
        subscribed: Maybe<Types.ChainLabel>,
        connection: Promise<Connection>
    }
}

export class Chains extends React.Component<Chains.Props, {}> {
    public render() {
        return (
            <div className="Chains">
                <Icon src={chainIcon} alt="Observed chain" />
                {
                    this.chains.map((chain) => this.renderChain(chain))
                }
            </div>
        );
    }

    private renderChain(chain: Types.ChainLabel): React.ReactNode {
        const className = chain === this.props.subscribed
            ? 'Chains-chain Chains-chain-selected'
            : 'Chains-chain';

        return (
            <a key={chain} className={className} onClick={this.subscribe.bind(this, chain)}>
                {chain}
            </a>
        )
    }

    private get chains(): Types.ChainLabel[] {
        return Array.from(this.props.chains);
    }

    private async subscribe(chain: Types.ChainLabel) {
        const connection = await this.props.connection;

        connection.subscribe(chain);
    }
}
