import * as React from 'react';
import { Tooltip } from '../';

export namespace Truncate {
  export interface Props {
    text: string;
    chars?: number;
    copy?: boolean;
    position?: Tooltip.Props['position'];
  }
}

export class Truncate extends React.Component<Truncate.Props, {}> {
  public render() {
    const { text, position, copy, chars } = this.props;

    if (!text) {
      return '-';
    }

    if (chars != null && text.length <= chars) {
      return text;
    }

    const truncated = chars ? (
      `${text.substr(0, chars - 1)}…`
    ) : (
      <div className="Column-truncate">{text}</div>
    );

    return (
      <>
        <Tooltip
          text={text}
          position={position}
          copy={copy}
          className="Column-Tooltip"
        />
        {truncated}
      </>
    );
  }

  public shouldComponentUpdate(nextProps: Truncate.Props): boolean {
    return (
      this.props.text !== nextProps.text ||
      this.props.position !== nextProps.position
    );
  }
}
