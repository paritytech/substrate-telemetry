import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import { Maybe } from './maybe';
import { NodeId, getId } from './nodeId';
import { parseMessage, getBestBlock, Message, BestBlock } from './message';

const BLOCK_TIME_HISTORY = 10;

export default class Node extends EventEmitter {
    public id: NodeId;
    public name: string;
    public implementation: string;
    public version: string;
    public height: number = 0;
    public config: string;
    public latency: number = 0;
    public blockTime: number = 0;

    private socket: WebSocket;
    private blockTimes: Array<number> = new Array(BLOCK_TIME_HISTORY);
    private lastBlockAt: Maybe<Date> = null;

    constructor(socket: WebSocket, name: string, config: string, implentation: string, version: string) {
        super();

        this.id = getId();
        this.socket = socket;
        this.name = name;
        this.config = config;
        this.implementation = implentation;
        this.version = version;

        console.log(`Listening to a new node: ${name}`);

        socket.on('message', (data: WebSocket.Data) => {
            const message = parseMessage(data);

            if (!message) return;

            this.updateLatency(message.ts);

            // console.log('received', message);

            const update = getBestBlock(message);

            if (update) {
                this.updateBestBlock(update);
            }
        });

        socket.on('close', () => this.disconnect());
        socket.on('error', () => this.disconnect());
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

    private disconnect() {
        console.log(`${this.name} has disconnected`);

        this.socket.removeAllListeners('message');
        this.emit('disconnect');
    }

    private updateLatency(time: Date) {
        this.latency = Date.now() - +time;
    }

    private updateBestBlock(update: BestBlock) {
        const { height, ts: time, best } = update;

        if (this.height < height) {
            const blockTime = this.getBlockTime(time);

            this.height = height;
            this.lastBlockAt = time;
            this.blockTimes[height % BLOCK_TIME_HISTORY] = blockTime;
            this.blockTime = blockTime;

            this.emit('block');
        }
    }

    private getBlockTime(time: Date): number {
        if (!this.lastBlockAt) {
            return 0;
        }

        return +time - +this.lastBlockAt;
    }

    get average(): number {
        let accounted = 0;
        let sum = 0;

        for (const time of this.blockTimes) {
            if (time) {
                accounted += 1;
                sum += time;
            }
        }

        if (accounted === 0) {
            return 0;
        }

        return sum / accounted;
    }
}
