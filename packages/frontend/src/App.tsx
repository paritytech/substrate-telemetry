import * as React from 'react';
import { Types } from '@dotstats/common';
import { Node } from './Node';
import { Icon } from './Icon';
import { Connection } from './message';
import { State } from './state';
import { formatNumber } from './utils';

import './App.css';
import nodeIcon from './icons/server.svg';
import nodeTypeIcon from './icons/terminal.svg';
import peersIcon from './icons/broadcast.svg';
import transactionsIcon from './icons/inbox.svg';
import blockIcon from './icons/package.svg';
import blockHashIcon from './icons/file-binary.svg';
import blockTimeIcon from './icons/history.svg';

export default class App extends React.Component<{}, State> {
    public state: State = {
        best: 0 as Types.BlockNumber,
        nodes: new Map()
    };

    constructor(props: {}) {
        super(props);

        this.connect();
    }

    public render() {
        return (
            <div className="App">
                <div className="App-header">
                    <Icon src={blockIcon} alt="Best Block" /> #{formatNumber(this.state.best)}
                </div>
                <table className="App-list">
                    <thead>
                        <tr>
                            <th><Icon src={nodeIcon} alt="Node" /></th>
                            <th><Icon src={nodeTypeIcon} alt="Implementation" /></th>
                            <th><Icon src={peersIcon} alt="Peer Count" /></th>
                            <th><Icon src={transactionsIcon} alt="Transactions in Queue" /></th>
                            <th><Icon src={blockIcon} alt="Block" /></th>
                            <th><Icon src={blockHashIcon} alt="Block Hash" /></th>
                            <th><Icon src={blockTimeIcon} alt="Block Time" /></th>
                        </tr>
                    </thead>
                    <tbody>
                    {
                        this.nodes().map((props) => <Node key={props.id} {...props} />)
                    }
                    </tbody>
                </table>
            </div>
        );
    }

    private async connect() {
        Connection.create((changes) => {
            if (changes) {
                this.setState(changes);
            }

            return this.state;
        });
    }

    private nodes(): Node.Props[] {
        return Array.from(this.state.nodes.values());
    }
}
