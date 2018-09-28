import * as React from 'react';

import './Tooltip.css';

export namespace Tooltip {
  export interface Props {
    text: string;
    inline?: boolean;
    className?: string;
    position?: 'left' | 'right' | 'center';
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
    const { text, inline, className, position } = this.props;

    let containerClass = 'Tooltip-container';
    let tooltipClass = 'Tooltip';

    if (className) {
      containerClass += ' ' + className;
    }

    if (inline) {
      containerClass += ' Tooltip-container-inline';
    }

    if (position && position !== 'center') {
      tooltipClass += ` Tooltip-${position}`;
    }

    return (
      <div className={containerClass}>
        <div className={tooltipClass} ref={this.onRef}>{text}</div>
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
