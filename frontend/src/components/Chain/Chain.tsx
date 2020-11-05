import * as React from 'react';
import { Connection } from '../../Connection';
import { Types, Maybe } from '../../common';
import { State as AppState, Update as AppUpdate } from '../../state';
import { getHashData } from '../../utils';
import { Header } from './';
import { Tile, Ago, List, Map, Settings, Consensus } from '../';
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
      case 'consensus':
        display = 'consensus';
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

    if (display === 'consensus') {
      return <Consensus appState={appState} connection={connection} />;
    }

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
