import { timestamp, FeedMessage, Types, Maybe, sleep } from '@dotstats/common';
import { State, Update } from './state';

const { Actions } = FeedMessage;

const TIMEOUT_BASE = (1000 * 5) as Types.Milliseconds; // 5 seconds
const TIMEOUT_MAX = (1000 * 60 * 5) as Types.Milliseconds; // 5 minutes

export class Connection {
    public static async create(update: Update): Promise<Connection> {
        return new Connection(await Connection.socket(), update);
    }

    private static readonly address = `ws://${window.location.hostname}:8080`;

    private static async socket(): Promise<WebSocket> {
        let socket = await Connection.trySocket();
        let timeout = TIMEOUT_BASE;

        while (!socket) {
            await sleep(timeout);

            timeout = Math.max(timeout * 2, TIMEOUT_MAX) as Types.Milliseconds;
            socket = await Connection.trySocket();
        }

        return socket;
    }

    private static async trySocket(): Promise<Maybe<WebSocket>> {
        return new Promise<Maybe<WebSocket>>((resolve, _) => {
            function clean() {
                socket.removeEventListener('open', onSuccess);
                socket.removeEventListener('close', onFailure);
                socket.removeEventListener('error', onFailure);
            }

            function onSuccess() {
                clean();
                resolve(socket);
            }

            function onFailure() {
                clean();
                resolve(null);
            }

            const socket = new WebSocket(Connection.address);

            socket.addEventListener('open', onSuccess);
            socket.addEventListener('error', onFailure);
            socket.addEventListener('close', onFailure);
        });
    }

    private socket: WebSocket;
    private state: Readonly<State>;
    private readonly update: Update;

    constructor(socket: WebSocket, update: Update) {
        this.socket = socket;
        this.update = update;
        this.bindSocket();
    }

    private bindSocket() {
        this.state = this.update({ nodes: new Map() });
        this.socket.addEventListener('message', this.handleMessages);
        this.socket.addEventListener('close', this.handleDisconnect);
        this.socket.addEventListener('error', this.handleDisconnect);
    }

    private clean() {
        this.socket.removeEventListener('message', this.handleMessages);
        this.socket.removeEventListener('close', this.handleDisconnect);
        this.socket.removeEventListener('error', this.handleDisconnect);
    }

    private handleMessages = (event: MessageEvent) => {
        const data = event.data as FeedMessage.Data;
        const nodes = this.state.nodes;
        const changes = { nodes };

        messages: for (const message of FeedMessage.deserialize(data)) {
            switch (message.action) {
                case Actions.BestBlock: {
                    const [best, blockTimestamp] = message.payload;

                    this.state = this.update({ best, blockTimestamp });

                    continue messages;
                }

                case Actions.AddedNode: {
                    const [id, nodeDetails, nodeStats, blockDetails] = message.payload;
                    const node = { id, nodeDetails, nodeStats, blockDetails };

                    nodes.set(id, node);

                    break;
                }

                case Actions.RemovedNode: {
                    nodes.delete(message.payload);

                    break;
                }

                case Actions.ImportedBlock: {
                    const [id, blockDetails] = message.payload;
                    const node = nodes.get(id);

                    if (!node) {
                        return;
                    }

                    node.blockDetails = blockDetails;

                    break;
                }

                case Actions.NodeStats: {
                    const [id, nodeStats] = message.payload;
                    const node = nodes.get(id);

                    if (!node) {
                        return;
                    }

                    node.nodeStats = nodeStats;

                    break;
                }

                case Actions.TimeSync: {
                    this.state = this.update({
                        timeDiff: (timestamp() - message.payload) as Types.Milliseconds
                    });

                    continue messages;
                }

                default: {
                    continue messages;
                }
            }
        }

        this.state = this.update(changes);
    }

    private handleDisconnect = async () => {
        this.clean();
        this.socket.close();
        this.socket = await Connection.socket();
        this.bindSocket();
    }
}
