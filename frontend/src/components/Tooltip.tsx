import * as React from 'react';
import { Maybe } from '../common';

import './Tooltip.css';

export namespace Tooltip {
  export interface Props {
    text: string;
    copy?: (cb: CopyCallback) => void;
    className?: string;
    position?: 'left' | 'right' | 'center';
    onInit?: (update: UpdateCallback) => void;
  }

  export interface State {
    copied: boolean;
  }

  export type UpdateCallback = (text: string) => void;
  export type CopyCallback = Maybe<() => void>;
}

function copyToClipboard(text: string) {
  const el = document.createElement('textarea');
  el.value = text;
  document.body.appendChild(el);
  el.select();
  document.execCommand('copy');
  document.body.removeChild(el);
}

export class Tooltip extends React.Component<Tooltip.Props, Tooltip.State> {
  public state = { copied: false };

  private el: HTMLDivElement;
  private timer: NodeJS.Timer | null = null;

  public componentDidMount() {
    if (this.props.onInit) {
      this.props.onInit(this.update);
    }
    if (this.props.copy) {
      this.props.copy(this.onClick);
    }
  }

  public componentWillUnmount() {
    if (this.timer) {
      clearTimeout(this.timer);
    }
    if (this.props.copy) {
      this.props.copy(null);
    }
  }

  public shouldComponentUpdate(
    nextProps: Tooltip.Props,
    nextState: Tooltip.State
  ) {
    return (
      this.props.text !== nextProps.text ||
      this.state.copied !== nextState.copied
    );
  }

  public render() {
    const { text, className, position } = this.props;
    const { copied } = this.state;

    let tooltipClass = 'Tooltip';

    if (position && position !== 'center') {
      tooltipClass += ` Tooltip-${position}`;
    }

    if (copied) {
      tooltipClass += ' Tooltip-copied';
    }

    return (
      <div className={tooltipClass} ref={this.onRef}>
        {copied ? 'Copied to clipboard!' : text}
      </div>
    );
  }

  private onRef = (el: HTMLDivElement) => {
    this.el = el;
  };

  private update = (text: string) => {
    this.el.textContent = text;
  };

  private onClick = () => {
    copyToClipboard(this.props.text);

    if (this.timer) {
      clearTimeout(this.timer);
    }

    this.setState({ copied: true });
    this.timer = setTimeout(this.restore, 2000);
  };

  private restore = () => {
    this.setState({ copied: false });
    this.timer = null;
  };
}
