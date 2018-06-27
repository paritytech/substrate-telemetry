import { Data } from 'ws';
import { Maybe, Opaque } from './shared';

export function parseMessage(data: Data): Maybe<Message> {
    try {
        const message = JSON.parse(data.toString());

        if (message && typeof message.msg === 'string' && typeof message.ts === 'string') {
            message.ts = new Date(message.ts);

            return message;
        }
    } catch (_) {
        console.warn('Error parsing message JSON');
    }

    return null;
}

export function getBestBlock(message: Message): Maybe<BestBlock> {
    switch (message.msg) {
        case 'node.start':
        case 'system.interval':
        case 'block.import':
            return message;
        default:
            return null;
    }
}

interface MessageBase {
    ts: Date,
    level: 'INFO' | 'WARN',
}

export interface BestBlock {
    best: string,
    height: number,
    ts: Date,
}

interface SystemConnected {
    msg: 'system.connected',
    name: string,
    chain: string,
    config: string,
    implementation: string,
    version: string,
}

interface SystemInterval extends BestBlock {
    msg: 'system.interval',
    txcount: number,
    peers: number,
    status: 'Idle' | string, // TODO: 'Idle' | ...?
}

interface NodeStart extends BestBlock {
    msg: 'node.start',
}

interface BlockImport extends BestBlock {
    msg: 'block.import',
}

// Union type
export type Message = MessageBase & (
    SystemConnected |
    SystemInterval  |
    NodeStart       |
    BlockImport
);


// received: {"msg":"block.import","level":"INFO","ts":"2018-06-18T17:30:35.285406538+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"node.start","level":"INFO","ts":"2018-06-18T17:30:40.038731057+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"system.connected","level":"INFO","ts":"2018-06-18T17:30:40.038975471+02:00","chain":"dev","config":"","version":"0.2.0","implementation":"parity-polkadot","name":"Majestic Widget"}
// received: {"msg":"system.interval","level":"INFO","ts":"2018-06-19T14:00:05.091355364+02:00","txcount":0,"best":"360c9563857308703398f637932b7ffe884e5c7b09692600ff09a4d753c9d948","height":7559,"peers":0,"status":"Idle"}
