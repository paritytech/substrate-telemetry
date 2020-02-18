import * as React from 'react';
import { Tooltip } from '../';

export namespace Truncate {
  export interface Props {
    text: string;
    copy?: boolean;
    position?: Tooltip.Props['position'];
  }
}

export class Truncate extends React.Component<Truncate.Props, {}> {
  public render() {
    const { text, position, copy } = this.props;

    if (!text) {
      return '-';
    }

    return (
      <Tooltip text={text} position={position} copy={copy} className="Row-Tooltip">
        <div className="Row-truncate">{text}</div>
      </Tooltip>
    );
  }

  public shouldComponentUpdate(nextProps: Truncate.Props): boolean {
    return this.props.text !== nextProps.text || this.props.position !== nextProps.position;
  }
}
