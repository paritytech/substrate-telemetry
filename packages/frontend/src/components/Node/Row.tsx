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
  public shouldComponentUpdate(nextProps: any, nextState: any) {
    if (this.props.nodesPinned !== nextProps.nodesPinned) {
      return true;
    }
    return false;
  }

  public render() {
    const [name, implementation, version] = this.props.nodeDetails;
    const [height, hash, blockTime, blockTimestamp, propagationTime] = this.props.blockDetails;
    const [peers, txcount] = this.props.nodeStats;
    const { nodesPinned } = this.props;

    return (
      <tr>
        <td><span onClick={this.props.handleNodePinClick}><Icon src={heartIcon} alt="Pin Node" className={nodesPinned ? "IconRed" : "Icon"} /></span></td>
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
