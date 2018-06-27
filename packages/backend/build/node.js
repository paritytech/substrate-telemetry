"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const EventEmitter = require("events");
const common_1 = require("@dotstats/common");
const message_1 = require("./message");
const BLOCK_TIME_HISTORY = 10;
const TIMEOUT = 1000 * 60 * 5; // 5 seconds
const nextId = common_1.idGenerator();
class Node extends EventEmitter {
    constructor(socket, name, config, implentation, version) {
        super();
        this.height = 0;
        this.latency = 0;
        this.blockTime = 0;
        this.blockTimes = new Array(BLOCK_TIME_HISTORY);
        this.lastBlockAt = null;
        this.lastMessage = Date.now();
        this.id = nextId();
        this.socket = socket;
        this.name = name;
        this.config = config;
        this.implementation = implentation;
        this.version = version;
        console.log(`Listening to a new node: ${name}`);
        socket.on('message', (data) => {
            const message = message_1.parseMessage(data);
            if (!message)
                return;
            this.lastMessage = Date.now();
            this.updateLatency(message.ts);
            const update = message_1.getBestBlock(message);
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
    static fromSocket(socket) {
        return new Promise((resolve, reject) => {
            function cleanup() {
                clearTimeout(timeout);
                socket.removeAllListeners('message');
            }
            function handler(data) {
                const message = message_1.parseMessage(data);
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
    timeoutCheck(now) {
        if (this.lastMessage + TIMEOUT < now) {
            this.disconnect();
        }
    }
    nodeInfo() {
        return {
            name: this.name,
        };
    }
    blockInfo() {
        return {
            height: this.height,
            blockTime: this.blockTime,
        };
    }
    get average() {
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
    disconnect() {
        this.socket.removeAllListeners();
        this.socket.close();
        this.emit('disconnect');
    }
    updateLatency(time) {
        this.latency = this.lastMessage - +time;
    }
    updateBestBlock(update) {
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
    getBlockTime(time) {
        if (!this.lastBlockAt) {
            return 0;
        }
        return +time - +this.lastBlockAt;
    }
}
exports.default = Node;
//# sourceMappingURL=node.js.map