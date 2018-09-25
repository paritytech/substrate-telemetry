import * as React from 'react';
import { Types, Maybe } from '@dotstats/common';
import { State as AppState } from '../../state';
import { formatNumber, secondsWithPrecision, viewport } from '../../utils';
import { Tab, Filter } from './';
import { Tile, Node, Ago, Setting } from '../';
import { PersistentObject, PersistentSet } from '../../persist';

import blockIcon from '../../icons/package.svg';
import blockTimeIcon from '../../icons/history.svg';
import lastTimeIcon from '../../icons/watch.svg';
import listIcon from '../../icons/list-alt-regular.svg';
import worldIcon from '../../icons/location.svg';
import settingsIcon from '../../icons/settings.svg';

const MAP_RATIO = 800 / 350;
const MAP_HEIGHT_ADJUST = 400 / 350;
const HEADER = 148;

const ESCAPE_KEY = 27;
const BACKSPACE_KEY = 8;

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
    map: {
      width: number;
      height: number;
      top: number;
      left: number;
    }
  }
}

function sortNodes(a: AppState.Node, b: AppState.Node): number {
  if (a.pinned === b.pinned) {
    if (a.blockDetails[0] === b.blockDetails[0]) {
      const aPropagation = a.blockDetails[4] == null ? Infinity : a.blockDetails[4] as number;
      const bPropagation = b.blockDetails[4] == null ? Infinity : b.blockDetails[4] as number;

      // Ascending sort by propagation time
      return aPropagation - bPropagation;
    }
  } else {
    return Number(b.pinned) - Number(a.pinned);
  }

  // Descending sort by block number
  return b.blockDetails[0] - a.blockDetails[0];
}

export class Chain extends React.Component<Chain.Props, Chain.State> {
  constructor(props: Chain.Props) {
    super(props);

    let display: Chain.Display = 'list';

    switch (window.location.hash) {
      case '#map':
        display = 'map';
        break;
      case '#settings':
        display = 'settings';
        break;
    }

    this.state = {
      display,
      filter: null,
      map: {
        width: 0,
        height: 0,
        top: 0,
        left: 0
      }
    };
  }

  public componentWillMount() {
    this.calculateMapDimensions();

    window.addEventListener('resize', this.calculateMapDimensions);
    window.addEventListener('keyup', this.onKeyPress);
  }

  public componentWillUnmount() {
    window.removeEventListener('resize', this.calculateMapDimensions);
    window.removeEventListener('keyup', this.onKeyPress);
  }

  public render() {
    const { best, blockTimestamp, blockAverage } = this.props.appState;
    const currentTab = this.state.display;

    return (
      <div className="Chain">
        <div className="Chain-header">
          <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
          <Tile icon={blockTimeIcon} title="Average Time">{ blockAverage == null ? '-' : secondsWithPrecision(blockAverage / 1000) }</Tile>
          <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
          <div className="Chain-tabs">
            <Tab icon={listIcon} label="List" display="list" hash="" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={worldIcon} label="Map" display="map" hash="#map" current={currentTab} setDisplay={this.setDisplay} />
            <Tab icon={settingsIcon} label="Settings" display="settings" hash="#settings" current={currentTab} setDisplay={this.setDisplay} />
          </div>
        </div>
        <div className="Chain-content-container">
          <div className="Chain-content">
          {
            currentTab === 'list'
              ? this.renderList()
              : currentTab === 'map'
              ? this.renderMap()
              : this.renderSettings()
          }
          </div>
        </div>
      </div>
    );
  }

  private setDisplay = (display: Chain.Display) => {
    this.setState({ display });
  };

  private renderList() {
    const { settings } = this.props.appState;
    const { pins } = this.props;
    const { filter } = this.state;
    const nodeFilter = this.getNodeFilter();
    const nodes = nodeFilter ? this.nodes().filter(nodeFilter) : this.nodes();

    if (nodeFilter && nodes.length === 0) {
      return (
        <React.Fragment>
          {filter != null ? <Filter value={filter} onChange={this.onFilterChange} /> : null}
          <div className="Chain-no-nodes">¯\_(ツ)_/¯<br />Nothing matches</div>
        </React.Fragment>
      );
    }

    return (
      <React.Fragment>
        {filter != null ? <Filter value={filter} onChange={this.onFilterChange} /> : null}
        <table className="Chain-node-list">
          <Node.Row.Header settings={settings} />
          <tbody>
          {
            nodes
              .sort(sortNodes)
              .map((node) => <Node.Row key={node.id} node={node} settings={settings} pins={pins} />)
          }
          </tbody>
        </table>
      </React.Fragment>
    );
  }

  private renderMap() {
    const { filter } = this.state;
    const nodeFilter = this.getNodeFilter();

    return (
      <React.Fragment>
        {filter != null ? <Filter value={filter} onChange={this.onFilterChange} /> : null}
        <div className="Chain-map">
        {
          this.nodes().map((node) => {
            const location = node.location;
            const focused = nodeFilter == null || nodeFilter(node);

            if (!location || location[0] == null || location[1] == null) {
              // Skip nodes with unknown location
              return null;
            }

            const position = this.pixelPosition(location[0], location[1]);

            return (
              <Node.Location key={node.id} position={position} focused={focused} node={node} />
            );
          })
        }
        </div>
      </React.Fragment>
    );
  }

  private renderSettings() {
    const { settings } = this.props;

    return (
      <div className="Chain-settings">
        <div className="Chain-settings-category">
          <h2>Visible Columns</h2>
          {
            Node.Row.columns
              .map(({ label, icon, setting }, index) => {
                if (!setting) {
                  return null;
                }

                return <Setting key={index} setting={setting} settings={settings} icon={icon} label={label} />;
              })
          }
        </div>
      </div>
    );
  }

  private nodes(): AppState.Node[] {
    return Array.from(this.props.appState.nodes.values());
  }

  private pixelPosition(lat: Types.Latitude, lon: Types.Longitude): Node.Location.Position {
    const { map } = this.state;

    // Longitude ranges -180 (west) to +180 (east)
    // Latitude ranges +90 (north) to -90 (south)
    const left = Math.round(((180 + lon) / 360) * map.width + map.left);
    const top = Math.round(((90 - lat) / 180) * map.height + map.top) * MAP_HEIGHT_ADJUST;

    let quarter: Node.Location.Quarter = 0;

    if (lon > 0) {
      quarter = (quarter | 1) as Node.Location.Quarter;
    }

    if (lat < 0) {
      quarter = (quarter | 2) as Node.Location.Quarter;
    }

    return { left, top, quarter };
  }

  private calculateMapDimensions: () => void = () => {
    const vp = viewport();

    vp.width = Math.max(1350, vp.width);
    vp.height -= HEADER;

    const ratio = vp.width / vp.height;

    let top = 0;
    let left = 0;
    let width = 0;
    let height = 0;

    if (ratio >= MAP_RATIO) {
      width = Math.round(vp.height * MAP_RATIO);
      height = Math.round(vp.height);
      left = (vp.width - width) / 2;
    } else {
      width = Math.round(vp.width);
      height = Math.round(vp.width / MAP_RATIO);
      top = (vp.height - height) / 2;
    }

    this.setState({ map: { top, left, width, height }});
  }

  private onKeyPress = (event: KeyboardEvent) => {
    const { filter } = this.state;
    const key = event.key;
    const code = event.keyCode;

    const escape = filter != null && code === ESCAPE_KEY;
    const backspace = filter === '' && code === BACKSPACE_KEY;
    const singleChar = filter == null && key.length === 1;

    if (escape || backspace) {
      this.setState({ filter: null });
    } else if (singleChar) {
      this.setState({ filter: key });
    }
  }

  private onFilterChange = (filter: string) => {
    this.setState({ filter: filter.toLowerCase() });
  }

  private getNodeFilter(): Maybe<(node: AppState.Node) => boolean> {
    const { filter } = this.state;

    if (filter == null) {
      return null;
    }

    const filterLC = filter.toLowerCase();

    return ({ nodeDetails }) => nodeDetails[0].indexOf(filterLC) !== -1;
  }
}
