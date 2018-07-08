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
      <td>{implementation} v{version}</td>
      <td>{peers}</td>
      <td>{txcount}</td>
      <td>#{formatNumber(height)}</td>
      <td><span title={hash}>{trimHash(hash, 16)}</span></td>
      <td>{(blockTime / 1000).toFixed(3)}s</td>
      <td>{propagationTime === null ? '∞' : `${propagationTime}ms`}</td>
      <td><Ago when={blockTimestamp} /></td>
    </tr>
  );
}
