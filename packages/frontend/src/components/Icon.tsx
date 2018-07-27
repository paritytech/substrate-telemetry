import * as React from 'react';
import { Types } from '@dotstats/common';
import ReactSVG from 'react-svg';
import './Icon.css';

export interface Props {
  src: string;
  alt: string;
  className?: string;
  onClick?: () => void;
  nodeId?: Types.NodeId;
  isNodeIdPinned?: () => boolean;
};

export class Icon extends React.Component<{}, Props> {
  public props: Props;

  public shouldComponentUpdate(nextProps: any, nextState: any) {
    const { nodeId, isNodeIdPinned } = this.props;

    if (!nodeId || !nextProps.hasOwnProperty('isNodeIdPinned' || typeof nextProps.isNodeIdPinned === 'undefined')) {
      return false;
    }

    console.log('isNodeIdPinned vs nextProps.nodesPinned.get(nodeId)', isNodeIdPinned, nextProps.isNodeIdPinned, typeof nextProps.isNodeIdPinned === 'undefined');

    if (isNodeIdPinned !== nextProps.isNodeIdPinned) {
      return true;
    }

    return false;
  }

  public render() {
    const { alt, className, onClick, src, isNodeIdPinned } = this.props;

    return <ReactSVG title={alt} className={`${isNodeIdPinned ? 'IconRed' : 'Icon'} ${ className || '' }`} path={src} onClick={onClick} />;
  }
}
