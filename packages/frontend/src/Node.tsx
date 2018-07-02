import * as React from 'react';
import { formatNumber, trimHash } from './utils';
import { Types } from '@dotstats/common';

export namespace Node {
    export interface Props {
        id: Types.NodeId,
        nodeDetails: Types.NodeDetails,
        nodeStats: Types.NodeStats,
        blockDetails: Types.BlockDetails,
    }
}

export function Node(props: Node.Props) {
    const [name, implementation, version] = props.nodeDetails;
    const [height, hash, blockTime] = props.blockDetails;
    const [peers, txcount] = props.nodeStats;

    return (
        <tr>
            <td>{name}</td>
            <td>{implementation} v{version}</td>
            <td>{peers}</td>
            <td>{txcount}</td>
            <td>#{formatNumber(height)} / {trimHash(hash, 16)}</td>
            <td>{blockTime / 1000}s</td>
        </tr>
    );
}
