"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const EventEmitter = require("events");
const shared_1 = require("@dotstats/shared");
const nextId = shared_1.idGenerator();
function serialize(msg) {
    return JSON.stringify(msg);
}
class Feed extends EventEmitter {
    constructor(socket) {
        super();
        this.id = nextId();
        this.socket = socket;
        socket.on('error', () => this.disconnect());
        socket.on('close', () => this.disconnect());
    }
    static bestBlock(height) {
        return serialize({
            action: 'best',
            payload: height
        });
    }
    static addedNode(node) {
        return serialize({
            action: 'added',
            payload: [node.id, node.nodeInfo(), node.blockInfo()]
        });
    }
    static removedNode(node) {
        return serialize({
            action: 'removed',
            payload: node.id
        });
    }
    static imported(node) {
        return serialize({
            action: 'imported',
            payload: [node.id, node.blockInfo()]
        });
    }
    send(data) {
        this.socket.send(data);
    }
    disconnect() {
        this.socket.removeAllListeners();
        this.socket.close();
        this.emit('disconnect');
    }
}
exports.default = Feed;
//# sourceMappingURL=feed.js.map