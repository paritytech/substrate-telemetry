import * as React from 'react';
import './App.css';
import { Id } from '@dotstats/common';

export interface NodeInfo {
    name: string;
}

export interface BlockInfo {
    height: number;
    blockTime: number;
}

interface BestBlock {
    action: 'best';
    payload: number;
}

interface AddedNode {
    action: 'added';
    payload: [Id<Node>, NodeInfo, BlockInfo];
}

interface RemovedNode {
    action: 'removed';
    payload: Id<Node>;
}

interface Imported {
    action: 'imported';
    payload: [Id<Node>, BlockInfo];
}

type Message = BestBlock | AddedNode | RemovedNode | Imported;

interface Node {
    nodeInfo: NodeInfo,
    blockInfo: BlockInfo,
}

interface State {
    best: number,
    nodes: Map<Id<Node>, Node>
}

export default class App extends React.Component<{}, State> {
    public state: State = {
        best: 0,
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
                                <td>{node.nodeInfo.name}</td>
                                <td>{node.blockInfo.height}</td>
                                <td>{node.blockInfo.blockTime / 1000}s</td>
                            </tr>
                        ))
                    }
                    </tbody>
                </table>
            </div>
        );
    }

    private nodes(): Array<[Id<Node>, Node]> {
        return Array.from(this.state.nodes.entries());
    }

    private onMessage(message: Message) {
        const { nodes } = this.state;

        switch (message.action) {
            case 'best': {
                this.setState({ best: message.payload });
            }
            return;
            case 'added': {
                const [id, nodeInfo, blockInfo] = message.payload;
                const node = { nodeInfo, blockInfo };

                nodes.set(id, node);
            }
            break;
            case 'removed': {
                nodes.delete(message.payload);
            }
            break;
            case 'imported': {
                const [id, blockInfo] = message.payload;

                const node = nodes.get(id);

                if (!node) {
                    return;
                }

                node.blockInfo = blockInfo;
            }
            break;
            default:
            return;
        }

        this.setState({ nodes });
    }
}
