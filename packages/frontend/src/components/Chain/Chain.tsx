import * as React from 'react';
import { Connection } from '../../Connection';
import { Types, Maybe } from '@dotstats/common';
import { State as AppState } from '../../state';
import { formatNumber, secondsWithPrecision, getHashData } from '../../utils';
import { Tab } from './';
import { Tile, Ago, List, Map, Settings, Consensus, Stats } from '../';
import { Persistent, PersistentObject, PersistentSet } from '../../persist';

import blockIcon from '../../icons/cube.svg';
import finalizedIcon from '../../icons/cube-alt.svg';
import blockTimeIcon from '../../icons/history.svg';
import lastTimeIcon from '../../icons/watch.svg';
import listIcon from '../../icons/list-alt-regular.svg';
import worldIcon from '../../icons/location.svg';
import settingsIcon from '../../icons/settings.svg';
import consensusIcon from '../../icons/cube-alt.svg';
import statsIcon from '../../icons/piechart.svg';

import './Chain.css';

export namespace Chain {
  export type Display = 'list' | 'map' | 'settings' | 'consensus' | 'stats';

  export interface Props {
    appState: Readonly<AppState>;
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
      case 'stats':
        display = 'stats';
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
        <div className="Chain-header">
          <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
          <Tile icon={finalizedIcon} title="Finalized Block">#{formatNumber(finalized)}</Tile>
          <Tile icon={blockTimeIcon} title="Average Time">{ blockAverage == null ? '-' : secondsWithPrecision(blockAverage / 1000) }</Tile>
          <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
          <div className="Chain-tabs">
            <Tab icon={listIcon} label="List" display="list" tab="" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={worldIcon} label="Map" display="map" tab="map" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={statsIcon} label="Map" display="stats" tab="stats" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={consensusIcon} label="Consensus" display="consensus" tab="consensus" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={settingsIcon} label="Settings" display="settings" tab="settings" current={currentTab} setDisplay={this.setDisplay} />
          </div>
        </div>
        <div className="Chain-content-container">
          <div className="Chain-content">
            {this.renderContent()}
          </div>
        </div>
      </div>
    );
  }

  private renderContent() {
    const { display } = this.state;
    const { appState, settings, connection, pins, sortBy } = this.props;

    switch (display) {
      case 'settings':
        return (
          <Settings settings={settings} />
        );
      case 'stats':
        return (
          <Stats nodeVersions={appState.nodeVersions} />
        );
      case 'consensus':
        return (
          <Consensus appState={appState} connection={connection} />
        );
      case 'list':
        return (
          <List appState={appState} pins={pins} sortBy={sortBy} />
        );
      default:
        return (
          <Map appState={appState} />
        );
    }
  }

  private setDisplay = (display: Chain.Display) => {
    this.setState({ display });
  };
}
