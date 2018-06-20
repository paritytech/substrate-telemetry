import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import { Maybe } from './maybe';
import { parseMessage, getBestBlock, Message, BestBlock } from './message';

const BLOCK_TIME_HISTORY = 10;

export default class Node extends EventEmitter {
    private id: number;
    private socket: WebSocket;
    private name: string;
    private config: string;
    private implementation: string;
    private version: string;
    private height: number = 0;
    private blockTimes: Array<number> = new Array(BLOCK_TIME_HISTORY);
    private lastBlockTime: Maybe<Date> = null;
    private latency: number = 0;

    constructor(socket: WebSocket, name: string, config: string, implentation: string, version: string) {
        super();

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
            this.lastBlockTime = time;
            this.blockTimes[height % BLOCK_TIME_HISTORY] = blockTime;

            console.log(`Best block for ${this.name} is ${this.height}, block time: ${blockTime / 1000}s, average: ${this.average / 1000}s | latency ${this.latency}`);
        }
    }

    private getBlockTime(time: Date): number {
        if (!this.lastBlockTime) {
            return 0;
        }

        return +time - +this.lastBlockTime;
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
