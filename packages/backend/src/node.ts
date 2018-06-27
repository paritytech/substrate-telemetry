import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import { Maybe, Id, idGenerator } from '@dotstats/common';
import { parseMessage, getBestBlock, Message, BestBlock } from './message';

const BLOCK_TIME_HISTORY = 10;
const TIMEOUT = 1000 * 60 * 5; // 5 seconds

const nextId = idGenerator<Node>();

export interface NodeInfo {
    name: string;
}

export interface BlockInfo {
    height: number;
    blockTime: number;
}

export default class Node extends EventEmitter {
    public lastMessage: number;
    public id: Id<Node>;
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

        this.lastMessage = Date.now();
        this.id = nextId();
        this.socket = socket;
        this.name = name;
        this.config = config;
        this.implementation = implentation;
        this.version = version;

        console.log(`Listening to a new node: ${name}`);

        socket.on('message', (data) => {
            const message = parseMessage(data);

            if (!message) return;

            this.lastMessage = Date.now();
            this.updateLatency(message.ts);

            const update = getBestBlock(message);

            if (update) {
                this.updateBestBlock(update);
            }
        });

        socket.on('close', () => {
            console.log(`${this.name} has disconnected`);

            this.disconnect();
        });

        socket.on('error', (error) => {
            console.error(`${this.name} has errored`, error);

            this.disconnect();
        });
    }

    public static fromSocket(socket: WebSocket): Promise<Node> {
        return new Promise((resolve, reject) => {
            function cleanup() {
                clearTimeout(timeout);
                socket.removeAllListeners('message');
            }

            function handler(data: WebSocket.Data) {
                const message = parseMessage(data);

                if (message && message.msg === "system.connected") {
                    cleanup();

                    const { name, config, implementation, version } = message;

                    resolve(new Node(socket, name, config, implementation, version));
                }
            }

            socket.on('message', handler);

            const timeout = setTimeout(() => {
                cleanup();

                socket.close();

                return reject(new Error('Timeout on waiting for system.connected message'));
            }, 5000);
        });
    }

    public timeoutCheck(now: number) {
        if (this.lastMessage + TIMEOUT < now) {
            this.disconnect();
        }
    }

    public nodeInfo(): NodeInfo {
        return {
            name: this.name,
        };
    }

    public blockInfo(): BlockInfo {
        return {
            height: this.height,
            blockTime: this.blockTime,
        };
    }

    public get average(): number {
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



    private disconnect() {
        this.socket.removeAllListeners();
        this.socket.close();

        this.emit('disconnect');
    }

    private updateLatency(time: Date) {
        this.latency = this.lastMessage - +time;
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
}
