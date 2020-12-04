import * as React from 'react';

export namespace Truncate {
  export interface Props {
    text: string;
    chars?: number;
  }
}

export class Truncate extends React.Component<Truncate.Props, {}> {
  public shouldComponentUpdate(nextProps: Truncate.Props): boolean {
    return this.props.text !== nextProps.text;
  }

  public render() {
    const { text, chars } = this.props;

    if (!text) {
      return '-';
    }

    if (chars != null && text.length <= chars) {
      return text;
    }

    return chars ? (
      `${text.substr(0, chars - 1)}…`
    ) : (
      <div className="Column-truncate">{text}</div>
    );
  }
}
