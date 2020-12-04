import * as React from 'react';
import { Maybe } from '../common';
import { Node } from '../state';
import { Icon } from './';

import searchIcon from '../icons/search.svg';

import './Filter.css';

export namespace Filter {
  export interface Props {
    onChange: (value: Maybe<(node: Node) => boolean>) => void;
  }

  export interface State {
    value: string;
  }
}

const ESCAPE_KEY = 27;

export class Filter extends React.Component<Filter.Props, {}> {
  public state = {
    value: '',
  };

  private filterInput: HTMLInputElement;

  public componentDidMount() {
    window.addEventListener('keyup', this.onWindowKeyUp);
  }

  public componentWillUnmount() {
    window.removeEventListener('keyup', this.onWindowKeyUp);
  }

  public shouldComponentUpdate(
    nextProps: Filter.Props,
    nextState: Filter.State
  ): boolean {
    return (
      this.props.onChange !== nextProps.onChange ||
      this.state.value !== nextState.value
    );
  }

  public render() {
    const { value } = this.state;

    let className = 'Filter';

    if (value === '') {
      className += ' Filter-hidden';
    }

    return (
      <div className={className}>
        <Icon src={searchIcon} />
        <input
          ref={this.onRef}
          value={value}
          onChange={this.onChange}
          onKeyUp={this.onKeyUp}
        />
      </div>
    );
  }

  private setValue(value: string) {
    this.setState({ value });

    this.props.onChange(this.getNodeFilter(value));
  }

  private onRef = (el: HTMLInputElement) => {
    this.filterInput = el;
  };

  private onChange = () => {
    this.setValue(this.filterInput.value);
  };

  private onKeyUp = (event: React.KeyboardEvent<HTMLInputElement>) => {
    event.stopPropagation();

    if (event.keyCode === ESCAPE_KEY) {
      this.setValue('');
    }
  };

  private onWindowKeyUp = (event: KeyboardEvent) => {
    if (event.ctrlKey) {
      return;
    }

    const { value } = this.state;
    const key = event.key;

    const escape = value && event.keyCode === ESCAPE_KEY;
    const singleChar = value === '' && key.length === 1;

    if (escape) {
      this.setValue('');
    } else if (singleChar) {
      this.setValue(key);
      this.filterInput.focus();
    }
  };

  private getNodeFilter(value: string): Maybe<(node: Node) => boolean> {
    if (value === '') {
      return null;
    }

    const filter = value.toLowerCase();

    return ({ name, city }) => {
      const matchesName = name.toLowerCase().indexOf(filter) !== -1;
      const matchesCity =
        city != null && city.toLowerCase().indexOf(filter) !== -1;

      return matchesName || matchesCity;
    };
  }
}
