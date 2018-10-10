import * as React from 'react';
import { Maybe } from '@dotstats/common';
import { Icon } from '../';

import searchIcon from '../../icons/search.svg';

import './Filter.css';

export namespace Filter {
  export interface Props {
    value: Maybe<string>;
    onChange: (value: Maybe<string>) => void;
  }
}

const ESCAPE_KEY = 27;

export class Filter extends React.Component<Filter.Props, {}> {
  private filterInput: HTMLInputElement;
  private timer: NodeJS.Timer;

  public componentDidMount() {
    this.filterInput.focus();
  }

  public componentWillUnmout() {
    clearTimeout(this.timer);
  }

  public shouldComponentUpdate(nextProps: Filter.Props): boolean {
    if (this.props.value == null) {
      this.filterInput.focus();
    }

    if (this.props.value === nextProps.value && this.props.onChange === nextProps.onChange) {
      return false;
    }

    return true;
  }

  public render() {
    const { value } = this.props;

    let className = "Filter";

    if (value == null) {
      className += " Filter-hidden";
    }

    return (
      <div className={className}>
        <Icon src={searchIcon} />
        <input ref={this.onRef} value={value || ''} onChange={this.onChange} onKeyUp={this.onKeyUp} onBlur={this.onBlur} />
      </div>
    );
  }

  private onRef = (el: HTMLInputElement) => {
    this.filterInput = el;
  }

  private onChange = () => {
    const { value } = this.filterInput;

    this.props.onChange(value === '' ? null : value);
  }

  private onKeyUp = (event: React.KeyboardEvent<HTMLInputElement>) => {
    event.stopPropagation();

    if (event.keyCode === ESCAPE_KEY) {
      this.props.onChange(null);
    }
  }

  private onBlur = (event: React.FocusEvent<HTMLInputElement>) => {
    if (this.props.value == null) {
      clearTimeout(this.timer);
      this.timer = setTimeout(() => this.filterInput.focus(), 50);
    }
  }
}
