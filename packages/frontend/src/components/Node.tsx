import * as React from 'react';
import { formatNumber, trimHash } from '../utils';
import { Ago } from './Ago';
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
  const [height, hash, blockTime, blockTimestamp, propagationTime] = props.blockDetails;
  const [peers, txcount] = props.nodeStats;

  return (
    <tr>
      <td>{name}</td>
      <td style={{ width: 240 }}>{implementation} v{version}</td>
      <td style={{ width: 26 }}>{peers}</td>
      <td style={{ width: 26 }}>{txcount}</td>
      <td style={{ width: 88 }}>#{formatNumber(height)}</td>
      <td style={{ width: 154 }}><span title={hash}>{trimHash(hash, 16)}</span></td>
      <td style={{ width: 80 }}>{(blockTime / 1000).toFixed(3)}s</td>
      <td style={{ width: 58 }}>{propagationTime === null ? '∞' : `${propagationTime}ms`}</td>
      <td style={{ width: 82 }}><Ago when={blockTimestamp} /></td>
    </tr>
  );
}
