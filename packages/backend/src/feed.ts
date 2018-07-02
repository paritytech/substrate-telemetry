import * as WebSocket from 'ws';
import * as EventEmitter from 'events';
import Node from './node';
import { Opaque, FeedMessage, Types, idGenerator } from '@dotstats/common';

const nextId = idGenerator<Types.FeedId>();
const { Actions } = FeedMessage;

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

    public static bestBlock(height: Types.BlockNumber): FeedMessage.Message {
        return {
            action: Actions.BestBlock,
            payload: height
        };
    }

    public static addedNode(node: Node): FeedMessage.Message {
        return {
            action: Actions.AddedNode,
            payload: [node.id, node.nodeDetails(), node.nodeStats(), node.blockDetails()]
        };
    }

    public static removedNode(node: Node): FeedMessage.Message {
        return {
            action: Actions.RemovedNode,
            payload: node.id
        };
    }

    public static imported(node: Node): FeedMessage.Message {
        return {
            action: Actions.ImportedBlock,
            payload: [node.id, node.blockDetails()]
        };
    }

    public static stats(node: Node): FeedMessage.Message {
        return {
            action: Actions.NodeStats,
            payload: [node.id, node.nodeStats()]
        };
    }

    public sendData(data: FeedMessage.Data) {
        this.socket.send(data);
    }

    public sendMessages(messages: Array<FeedMessage.Message>) {
        this.socket.send(FeedMessage.serialize(messages))
    }

    private disconnect() {
        this.socket.removeAllListeners();
        this.socket.close();

        this.emit('disconnect');
    }
}
