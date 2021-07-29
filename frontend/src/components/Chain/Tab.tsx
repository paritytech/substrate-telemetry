// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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
import { Chain } from './';
import { Icon } from '../';
import { setHashData } from '../../utils';

import './Tab.css';

export namespace Tab {
  export interface Props {
    label: string;
    icon: string;
    display: Chain.Display;
    current: string;
    tab: string;
    setDisplay: (display: Chain.Display) => void;
  }
}

export class Tab extends React.Component<Tab.Props, {}> {
  public render() {
    const { label, icon, display, current } = this.props;
    const highlight = display === current;
    const className = highlight ? 'Chain-Tab-on Chain-Tab' : 'Chain-Tab';

    return (
      <div className={className} onClick={this.onClick} title={label}>
        <Icon src={icon} />
      </div>
    );
  }

  private onClick = () => {
    const { tab, display, setDisplay } = this.props;
    setHashData({ tab });
    setDisplay(display);
  };
}
