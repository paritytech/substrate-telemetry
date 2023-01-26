// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import { Maybe } from '../common';
import { Node } from '../state';
import { Icon } from './';

import searchIcon from '../icons/search.svg';

import './Filter.css';

interface FilterProps {
  onChange: (value: Maybe<(node: Node) => boolean>) => void;
}

interface FilterState {
  value: string;
}

const ESCAPE_KEY = 27;

export class Filter extends React.Component<FilterProps, FilterState> {
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
    nextProps: FilterProps,
    nextState: FilterState
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
    // Ignore if control key is being pressed
    if (event.ctrlKey) {
      return;
    }
    // Ignore events dispatched to other elements that want to use it
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (['INPUT', 'TEXTAREA'].includes((event.target as any)?.tagName)) {
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
