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
import { Connection } from '../../Connection';
import { Types, Maybe } from '../../common';
import {
  State as AppState,
  Update as AppUpdate,
  StateSettings,
} from '../../state';
import { getHashData } from '../../utils';
import { Header } from './';
import { List, Map, Settings, Stats } from '../';
import { Persistent, PersistentObject, PersistentSet } from '../../persist';

import './Chain.css';

export type ChainDisplay = 'list' | 'map' | 'settings' | 'consensus' | 'stats';

interface ChainProps {
  appState: Readonly<AppState>;
  appUpdate: AppUpdate;
  connection: Promise<Connection>;
  settings: PersistentObject<StateSettings>;
  pins: PersistentSet<Types.NodeName>;
  sortBy: Persistent<Maybe<number>>;
}

interface ChainState {
  display: ChainDisplay;
}

export class Chain extends React.Component<ChainProps, ChainState> {
  constructor(props: ChainProps) {
    super(props);

    let display: ChainDisplay = 'list';

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

    const { appState, appUpdate, pins, sortBy } = this.props;

    if (display === 'list') {
      return (
        <List
          appState={appState}
          appUpdate={appUpdate}
          pins={pins}
          sortBy={sortBy}
        />
      );
    }

    if (display === 'map') {
      return <Map appState={appState} />;
    }

    if (display === 'stats') {
      return <Stats appState={appState} />;
    }

    throw new Error('invalid `display`: ${display}');
  }

  private setDisplay = (display: ChainDisplay) => {
    this.setState({ display });
  };
}
