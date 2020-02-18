import * as React from 'react';

import './Tooltip.css';

export namespace Tooltip {
  export interface Props {
    text: string;
    copy?: boolean;
    inline?: boolean;
    className?: string;
    position?: 'left' | 'right' | 'center';
    onInit?: (update: UpdateCallback) => void;
  }

  export interface State {
    copied: boolean;
  }

  export type UpdateCallback = (text: string) => void;
}

function copyToClipboard(text: string) {
  const el = document.createElement('textarea');
  el.value = text;
  document.body.appendChild(el);
  el.select();
  document.execCommand('copy');
  document.body.removeChild(el);
};

export class Tooltip extends React.Component<Tooltip.Props, Tooltip.State> {
  public state = { copied: false };

  private el: HTMLDivElement;
  private timer: NodeJS.Timer;

  public componentDidMount() {
    if (this.props.onInit) {
      this.props.onInit(this.update);
    }
  }

  public componentWillUnmount() {
    clearTimeout(this.timer);
  }

  public render() {
    const { text, inline, className, position } = this.props;
    const { copied } = this.state;

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

    if (copied) {
      tooltipClass += ' Tooltip-copied';
    }

    return (
      <div className={containerClass} onClick={this.onClick}>
        <div className={tooltipClass} ref={this.onRef}>{copied ? 'Copied to clipboard!' : text}</div>
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

  private onClick = (event: React.MouseEvent<HTMLDivElement>) => {
    if (this.props.copy !== true) {
      return;
    }

    copyToClipboard(this.props.text);

    event.stopPropagation();

    clearTimeout(this.timer);

    this.setState({ copied: true });
    this.timer = setTimeout(this.restore, 2000);
  }

  private restore = () => {
    this.setState({ copied: false });
  }
}
