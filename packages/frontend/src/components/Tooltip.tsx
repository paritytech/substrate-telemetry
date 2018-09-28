import * as React from 'react';

import './Tooltip.css';

export namespace Tooltip {
  export interface Props {
    text: string;
    className?: string;
    onInit?: (update: UpdateCallback) => void;
  }

  export type UpdateCallback = (text: string) => void;
}

export class Tooltip extends React.Component<Tooltip.Props, {}> {
  private el: HTMLDivElement;

  public componentDidMount() {
    if (this.props.onInit) {
      this.props.onInit(this.update);
    }
  }

  public render() {
    const { text } = this.props;

    let className = 'Tooltip-container';

    if (this.props.className) {
      className += ' ' + this.props.className;
    }

    return (
      <div className={className}>
        <div className="Tooltip" ref={this.onRef}>{text}</div>
        {this.props.children}
      </div>
    );
  }

  private onRef = (el: HTMLDivElement) => {
    this.el = el;
  }

  private update = (text: string) => {
    this.el.textContent = text;
  }
}
