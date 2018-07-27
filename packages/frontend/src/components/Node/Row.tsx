import * as React from 'react';
import { formatNumber, trimHash, milliOrSecond, secondsWithPrecision } from '../../utils';
import { Ago, Icon } from '../';
import { Props } from './';

import heartIcon from '../../icons/heart.svg';

interface PinState {
  nodesPinned: any;
}

interface PinHandler {
  handleNodePinClick: () => void;
}

export class Row extends React.Component<Props & PinState & PinHandler> {
  public render() {
    const { id, nodesPinned } = this.props;
    const [name, implementation, version] = this.props.nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = this.props.blockDetails;
    const [peers, txcount] = this.props.nodeStats;
    const isNodeIdPinned = () => {
      if (typeof id === 'undefined') {
        return false;
      }
      console.log('nodesPinned[id]', nodesPinned[id])
      return nodesPinned[id] || false;
    }

    console.log('id, nodesPinned', id, nodesPinned);

    return (
      <tr>
        <td><span onClick={this.props.handleNodePinClick}><Icon src={heartIcon} alt="Pin Node" nodeId={id} isNodeIdPinned={isNodeIdPinned()} /></span></td>
        <td>{name}</td>
        <td>{implementation} v{version}</td>
        <td>{peers}</td>
        <td>{txcount}</td>
        <td>#{formatNumber(height)}</td>
        <td><span title={hash}>{trimHash(hash, 16)}</span></td>
        <td>{secondsWithPrecision(blockTime/1000)}</td>
        <td>{propagationTime === null ? '∞' : milliOrSecond(propagationTime as number)}</td>
        <td><Ago when={blockTimestamp} /></td>
      </tr>
    );
  }
}
