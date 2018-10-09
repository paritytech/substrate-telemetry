import * as React from 'react';
import { Icon } from '../';

import searchIcon from '../../icons/search.svg';

import './Filter.css';

export namespace Filter {
  export interface Props {
    value: string;
    onChange: (value: string) => void;
  }
}

export class Filter extends React.Component<Filter.Props, {}> {
  private filterInput: HTMLInputElement;

  public componentDidMount() {
    this.filterInput.focus();
  }

  public render() {
    const { value } = this.props;

    return (
      <div className="Chain-Filter">
        <Icon src={searchIcon} />
        <input ref={this.onRef} value={value} onChange={this.onChange} />
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
