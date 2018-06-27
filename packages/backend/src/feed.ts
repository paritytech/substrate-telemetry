import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import Node from './node';
import { Opaque, Types, idGenerator } from '@dotstats/common';

const nextId = idGenerator<Types.FeedId>();

/**
 * Opaque data type to be sent to the feed. Passing through
 * strings means we can only serialize once, no matter how
 * many feed clients are listening in.
 */
export type FeedData = Opaque<string, Types.FeedMessage>;

function serialize(msg: Types.FeedMessage): FeedData {
    return JSON.stringify(msg) as FeedData;
}

export default class Feed extends EventEmitter {
    public id: Types.FeedId;

    private socket: WebSocket;

    constructor(socket: WebSocket) {
        super();

        this.id = nextId();
        this.socket = socket;

        socket.on('error', () => this.disconnect());
        socket.on('close', () => this.disconnect());
    }

    public static bestBlock(height: Types.BlockNumber): FeedData {
        return serialize({
            action: 'best',
            payload: height
        });
    }

    public static addedNode(node: Node): FeedData {
        return serialize({
            action: 'added',
            payload: [node.id, node.nodeDetails(), node.blockDetails()]
        })
    }

    public static removedNode(node: Node): FeedData {
        return serialize({
            action: 'removed',
            payload: node.id
        });
    }

    public static imported(node: Node): FeedData {
        return serialize({
            action: 'imported',
            payload: [node.id, node.blockDetails()]
        });
    }

    public send(data: FeedData) {
        this.socket.send(data);
    }

    private disconnect() {
        this.socket.removeAllListeners();
        this.socket.close();

        this.emit('disconnect');
    }
}
