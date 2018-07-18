import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../utils';
import { Ago } from './Ago';
import { Types, Maybe } from '@dotstats/common';

export namespace Node {
  export interface Props {
    id: Types.NodeId;
    nodeDetails: Types.NodeDetails;
    nodeStats: Types.NodeStats;
    blockDetails: Types.BlockDetails;
    location: Maybe<Types.NodeLocation>;
  }

  export interface PixelPosition {
    left: number;
    top: number;
  }

  export function Row(props: Props) {
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
        <td style={{ width: 80 }}>{secondsWithPrecision(blockTime/1000)}</td>
        <td style={{ width: 58 }}>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td>
        <td style={{ width: 82 }}><Ago when={blockTimestamp} /></td>
      </tr>
    );
  }

  export function Location(props: Props & PixelPosition) {
    const { left, top } = props;

    return (
      <span
        className="Chain-map-node"
        style={{ left, top }}
        title={props.nodeDetails[0]}
        data-location={JSON.stringify(props.location)}
      />
    );
  }
}
