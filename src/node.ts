import * as WebSocket from 'ws';
import { parseMessage, getBestBlock, Message, BestBlock } from './message';

export default class Node {
    private socket: WebSocket;
    private name: string;
    private config: string;
    private implementation: string;
    private version: string;
    private height: number = 0;

    constructor(socket: WebSocket, name: string, config: string, implentation: string, version: string) {
        this.socket = socket;
        this.name = name;
        this.config = config;
        this.implementation = implentation;
        this.version = version;

        console.log(`Listening to a new node: ${name}`);

        socket.on('message', (data: WebSocket.Data) => {
            const message = parseMessage(data);

            if (!message) return;

            // console.log('received', message);

            const update = getBestBlock(message);

            if (update) {
                this.updateBestBlock(update);
            }
        });
    }

    updateBestBlock(update: BestBlock) {
        if (this.height < update.height) {
            this.height = update.height;

            console.log(`Best block for ${this.name} is ${this.height}`);
        }
    }

    static fromSocket(socket: WebSocket): Promise<Node> {
        return new Promise((resolve, reject) => {
            function cleanup() {
                clearTimeout(timeout);
                socket.removeEventListener('message');
            }

            function handler(data: WebSocket.Data) {
                const message = parseMessage(data);

                if (!message) {
                    cleanup();

                    return reject(new Error('Invalid message'));
                }

                if (message.msg === "system.connected") {
                    cleanup();

                    const { name, config, implementation, version } = message;

                    resolve(new Node(socket, name, config, implementation, version));
                }
            }

            socket.on('message', handler);

            const timeout = setTimeout(() => {
                cleanup();

                return reject(new Error('Timeout on waiting for system.connected message'));
            }, 5000);
        });
    }
}
