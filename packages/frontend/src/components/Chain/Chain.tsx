import * as React from 'react';
import { Types, Maybe } from '@dotstats/common';
import { State as AppState, Node as NodeState } from '../../state';
import { formatNumber, secondsWithPrecision, getHashData } from '../../utils';
import { Tab, Filter } from './';
import { Tile, Ago, List, Map, Settings } from '../';
import { PersistentObject, PersistentSet } from '../../persist';

import blockIcon from '../../icons/package.svg';
import blockTimeIcon from '../../icons/history.svg';
import lastTimeIcon from '../../icons/watch.svg';
import listIcon from '../../icons/list-alt-regular.svg';
import worldIcon from '../../icons/location.svg';
import settingsIcon from '../../icons/settings.svg';

const ESCAPE_KEY = 27;

import './Chain.css';

export namespace Chain {
  export type Display = 'list' | 'map' | 'settings';

  export interface Props {
    appState: Readonly<AppState>;
    settings: PersistentObject<AppState.Settings>;
    pins: PersistentSet<Types.NodeName>;
  }

  export interface State {
    display: Display;
    filter: Maybe<string>;
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
      filter: null,
    };
  }

  public componentWillMount() {
    window.addEventListener('keyup', this.onKeyUp);
  }

  public componentWillUnmount() {
    window.removeEventListener('keyup', this.onKeyUp);
  }

  public render() {
    const { appState } = this.props;
    const { best, blockTimestamp, blockAverage } = appState;
    const { display: currentTab } = this.state;

    return (
      <div className="Chain">
        <div className="Chain-header">
          <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
          <Tile icon={blockTimeIcon} title="Average Time">{ blockAverage == null ? '-' : secondsWithPrecision(blockAverage / 1000) }</Tile>
          <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
          <div className="Chain-tabs">
            <Tab icon={listIcon} label="List" display="list" tab="" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={worldIcon} label="Map" display="map" tab="map" current={currentTab} setDisplay={this.setDisplay} />
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
    const { display, filter } = this.state;

    if (display === 'settings') {
      return <Settings settings={this.props.settings} />;
    }

    const { appState, pins } = this.props;

    return (
      <React.Fragment>
        <Filter value={filter} onChange={this.onFilterChange} />
        {
          display === 'list'
            ? <List filter={this.getNodeFilter()} appState={appState} pins={pins} />
            : <Map filter={this.getNodeFilter()} appState={appState} />
        }
      </React.Fragment>
    );
  }

  private setDisplay = (display: Chain.Display) => {
    this.setState({ display });
  };

  private onKeyUp = (event: KeyboardEvent) => {
    if (event.ctrlKey) {
      return;
    }

    const { filter } = this.state;
    const key = event.key;

    const escape = filter != null && event.keyCode === ESCAPE_KEY;
    const singleChar = filter == null && key.length === 1;

    if (escape) {
      this.setState({ filter: null });
    } else if (singleChar) {
      this.setState({ filter: key });
    }
  }

  private onFilterChange = (filter: string) => {
    this.setState({ filter });
  }

  private getNodeFilter(): Maybe<(node: NodeState) => boolean> {
    const { filter } = this.state;

    if (filter == null) {
      return null;
    }

    const filterLC = filter.toLowerCase();

    return ({ name, city }) => {
      const matchesName = name.toLowerCase().indexOf(filterLC) !== -1;
      const matchesCity = city != null && city.toLowerCase().indexOf(filterLC) !== -1;

      return matchesName || matchesCity;
    }
  }
}
