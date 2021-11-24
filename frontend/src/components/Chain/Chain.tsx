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
import { Connection } from '../../Connection';
import { Types, Maybe } from '../../common';
import { State as AppState, Update as AppUpdate } from '../../state';
import { getHashData } from '../../utils';
import { Header } from './';
import { List, Map, Settings } from '../';
import { Persistent, PersistentObject, PersistentSet } from '../../persist';

import './Chain.css';

export namespace Chain {
  export type Display = 'list' | 'map' | 'settings' | 'consensus';

  export interface Props {
    appState: Readonly<AppState>;
    appUpdate: AppUpdate;
    connection: Promise<Connection>;
    settings: PersistentObject<AppState.Settings>;
    pins: PersistentSet<Types.NodeName>;
    sortBy: Persistent<Maybe<number>>;
  }

  export interface State {
    display: Display;
  }
}

export class Chain extends React.Component<Chain.Props, Chain.State> {
  constructor(props: Chain.Props) {
    super(props);

    let display: Chain.Display = 'list';

    switch (getHashData().tab) {
      case 'map':
        display = 'map';
        break;
      case 'settings':
        display = 'settings';
        break;
    }

    this.state = {
      display,
    };
  }

  public render() {
    const { appState } = this.props;
    const { best, finalized, blockTimestamp, blockAverage } = appState;
    const { display: currentTab } = this.state;

    return (
      <div className="Chain">
        <Header
          best={best}
          finalized={finalized}
          blockAverage={blockAverage}
          blockTimestamp={blockTimestamp}
          currentTab={currentTab}
          setDisplay={this.setDisplay}
        />
        <div className="Chain-content-container">
          <div className="Chain-content">{this.renderContent()}</div>
        </div>
      </div>
    );
  }

  private renderContent() {
    const { display } = this.state;

    if (display === 'settings') {
      return <Settings settings={this.props.settings} />;
    }

    const { appState, appUpdate, connection, pins, sortBy } = this.props;

    return display === 'list' ? (
      <List
        appState={appState}
        appUpdate={appUpdate}
        pins={pins}
        sortBy={sortBy}
      />
    ) : (
      <Map appState={appState} />
    );
  }

  private setDisplay = (display: Chain.Display) => {
    this.setState({ display });
  };
}
