import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import Node, { NodeInfo, BlockInfo } from './node';
import { Opaque, Id, idGenerator } from './utils';

const nextId = idGenerator<Feed>();

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

/**
 * Opaque data type to be sent to the feed. Passing through
 * strings means we can only serialize once, no matter how
 * many feed clients are listening in.
 */
export type FeedData = Opaque<string, Message>;

function serialize(msg: Message): FeedData {
    return JSON.stringify(msg) as FeedData;
}

export default class Feed extends EventEmitter {
    public id: Id<Feed>;

    private socket: WebSocket;

    constructor(socket: WebSocket) {
        super();

        this.id = nextId();
        this.socket = socket;

        socket.on('error', () => this.disconnect());
        socket.on('close', () => this.disconnect());
    }

    public static bestBlock(height: number): FeedData {
        return serialize({
            action: 'best',
            payload: height
        });
    }

    public static addedNode(node: Node): FeedData {
        return serialize({
            action: 'added',
            payload: [node.id, node.nodeInfo(), node.blockInfo()]
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
            payload: [node.id, node.blockInfo()]
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
