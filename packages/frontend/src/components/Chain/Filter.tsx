import * as React from 'react';
import { Maybe } from '@dotstats/common';
import { Icon } from '../';

import searchIcon from '../../icons/search.svg';

import './Filter.css';

export namespace Filter {
  export interface Props {
    value: Maybe<string>;
    onChange: (value: string) => void;
  }
}

export class Filter extends React.Component<Filter.Props, {}> {
  private filterInput: HTMLInputElement;

  public componentDidMount() {
    this.filterInput.focus();
  }

  public shouldComponentUpdate(nextProps: Filter.Props): boolean {
    if (this.props.value === nextProps.value && this.props.onChange === nextProps.onChange) {
      return false;
    }

    if (this.props.value == null) {
      this.filterInput.focus();
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
        <input ref={this.onRef} value={value || ''} onChange={this.onChange} />
      </div>
    );
  }

  private onRef = (el: HTMLInputElement) => {
    this.filterInput = el;
  }

  private onChange = () => {
    const { value } = this.filterInput;

    this.props.onChange(value);
  }
}
