import { Opaque } from './helpers';
import { NodeId, NodeDetails, NodeStats, BlockNumber, BlockDetails } from './types';

export const Actions = {
    BestBlock: 0 as 0,
    AddedNode: 1 as 1,
    RemovedNode: 2 as 2,
    ImportedBlock: 3 as 3,
    NodeStats: 4 as 4,
};

export type Action = typeof Actions[keyof typeof Actions];
export type Payload = Message['payload'];

export namespace Variants {
    export interface MessageBase {
        action: Action;
    }

    export interface BestBlockMessage extends MessageBase {
        action: typeof Actions.BestBlock;
        payload: BlockNumber;
    }

    export interface AddedNodeMessage extends MessageBase {
        action: typeof Actions.AddedNode;
        payload: [NodeId, NodeDetails, NodeStats, BlockDetails];
    }

    export interface RemovedNodeMessage extends MessageBase {
        action: typeof Actions.RemovedNode;
        payload: NodeId;
    }

    export interface ImportedBlockMessage extends MessageBase {
        action: typeof Actions.ImportedBlock;
        payload: [NodeId, BlockDetails];
    }

    export interface NodeStatsMessage extends MessageBase {
        action: typeof Actions.NodeStats;
        payload: [NodeId, NodeStats];
    };
}

export type Message =
    | Variants.BestBlockMessage
    | Variants.AddedNodeMessage
    | Variants.RemovedNodeMessage
    | Variants.ImportedBlockMessage
    | Variants.NodeStatsMessage;

/**
 * Opaque data type to be sent to the feed. Passing through
 * strings means we can only serialize once, no matter how
 * many feed clients are listening in.
 */
export type Data = Opaque<string, 'FeedMessage.Data'>;

/**
 * Serialize an array of `Message`s to a single JSON string.
 *
 * All messages are squashed into a single array of alternating opcodes and payloads.
 *
 * Action `string`s are converted to opcodes using the `actionToCode` mapping.
 */
export function serialize(messages: Array<Message>): Data {
    const squashed = new Array(messages.length * 2);
    let index = 0;

    messages.forEach((message) => {
        const { action, payload } = message;

        squashed[index++] = action;
        squashed[index++] = payload;
    })

    return JSON.stringify(squashed) as Data;
}

/**
 * Deserialize data to an array of `Message`s.
 */
export function deserialize(data: Data): Array<Message> {
    const json: Array<Action | Payload> = JSON.parse(data);

    if (!Array.isArray(json) || json.length === 0 || json.length % 2 !== 0) {
        throw new Error('Invalid FeedMessage.Data');
    }

    const messages: Array<Message> = new Array(json.length / 2);

    for (const index of messages.keys()) {
        const [ action, payload ] = json.slice(index * 2);

        messages[index] = { action, payload } as Message;
    }

    return messages;
}
