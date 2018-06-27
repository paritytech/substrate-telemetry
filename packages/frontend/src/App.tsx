import * as React from 'react';
import './App.css';
import { Types } from '@dotstats/common';

interface Node {
    nodeDetails: Types.NodeDetails,
    blockDetails: Types.BlockDetails,
}

interface State {
    best: Types.BlockNumber,
    nodes: Map<Types.NodeId, Node>
}

export default class App extends React.Component<{}, State> {
    public state: State = {
        best: 0 as Types.BlockNumber,
        nodes: new Map()
    };

    constructor(props: {}) {
        super(props);

        const socket = new WebSocket(`ws://${window.location.hostname}:8080`);

        socket.addEventListener('message', ({ data }) => {
            this.onMessage(JSON.parse(data));
        });
    }

    public render() {
        return (
            <div className="App">
                <p>Best block: {this.state.best}</p>
                <table>
                    <thead>
                        <tr>
                            <th>Name</th><th>Block</th><th>Block time</th>
                        </tr>
                    </thead>
                    <tbody>
                    {
                        this.nodes().map(([ id, node ]) => (
                            <tr key={id}>
                                <td>{node.nodeDetails.name}</td>
                                <td>{node.blockDetails.height}</td>
                                <td>{node.blockDetails.blockTime / 1000}s</td>
                            </tr>
                        ))
                    }
                    </tbody>
                </table>
            </div>
        );
    }

    private nodes(): Array<[Types.NodeId, Node]> {
        return Array.from(this.state.nodes.entries());
    }

    private onMessage(message: Types.FeedMessage) {
        const { nodes } = this.state;

        switch (message.action) {
            case 'best': {
                this.setState({ best: message.payload });
            }
            return;
            case 'added': {
                const [id, nodeDetails, blockDetails] = message.payload;
                const node = { nodeDetails, blockDetails };

                nodes.set(id, node);
            }
            break;
            case 'removed': {
                nodes.delete(message.payload);
            }
            break;
            case 'imported': {
                const [id, blockDetails] = message.payload;

                const node = nodes.get(id);

                if (!node) {
                    return;
                }

                node.blockDetails = blockDetails;
            }
            break;
            default:
            return;
        }

        this.setState({ nodes });
    }
}
