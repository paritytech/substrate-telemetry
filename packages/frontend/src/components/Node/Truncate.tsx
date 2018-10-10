import * as React from 'react';
import { Tooltip } from '../';

namespace Truncate {
  export interface Props {
    text: string;
    copy?: boolean;
    position?: Tooltip.Props['position'];
  }
}

class Truncate extends React.Component<Truncate.Props, {}> {
  public render() {
    const { text, position, copy } = this.props;

    return (
      <Tooltip text={text} position={position} copy={copy} className="Node-Row-Tooltip">
        <div className="Node-Row-truncate">{text}</div>
      </Tooltip>
    );
  }

  public shouldComponentUpdate(nextProps: Truncate.Props): boolean {
    return this.props.text !== nextProps.text || this.props.position !== nextProps.position;
  }
}

export default Truncate;
