import * as React from 'react';
import { State as AppState } from '../state';
import { formatNumber, secondsWithPrecision, viewport } from '../utils';
import { Tile, Icon, Node, Ago } from './';
import { Types } from '@dotstats/common';

import nodeIcon from '../icons/server.svg';
import nodeTypeIcon from '../icons/terminal.svg';
import peersIcon from '../icons/broadcast.svg';
import transactionsIcon from '../icons/inbox.svg';
import blockIcon from '../icons/package.svg';
import blockHashIcon from '../icons/file-binary.svg';
import blockTimeIcon from '../icons/history.svg';
import propagationTimeIcon from '../icons/dashboard.svg';
import lastTimeIcon from '../icons/watch.svg';
import worldIcon from '../icons/globe.svg';

const MAP_RATIO = 495 / 266; // width / height
const HEADER = 148;

import './Chain.css';

export namespace Chain {
  export interface Props {
    appState: Readonly<AppState>;
  }

  export interface State {
    display: 'map' | 'table';
    map: {
      width: number;
      height: number;
      top: number;
      left: number;
    }
  }
}

function sortNodes(a: Node.Props, b: Node.Props): number {
  const aPropagation = a.blockDetails[4] == null ? Infinity : a.blockDetails[4] as number;
  const bPropagation = b.blockDetails[4] == null ? Infinity : b.blockDetails[4] as number;

  if (aPropagation === Infinity && bPropagation === Infinity) {
    // Descending sort by block number
    return b.blockDetails[0] - a.blockDetails[0];
  }

  // Ascending sort by propagation time
  return aPropagation - bPropagation;
}

export class Chain extends React.Component<Chain.Props, Chain.State> {
  constructor(props: Chain.Props) {
    super(props);

    this.state = {
      display: 'table',
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
  }

  public componentWillUnmount() {
    window.removeEventListener('resize', this.calculateMapDimensions);
  }

  public render() {
    const { best, blockTimestamp, blockAverage } = this.props.appState;
    const { display } = this.state;

    const toggleClass = ['Chain-map-toggle'];

    if (display === 'map') {
      toggleClass.push('Chain-map-toggle-on');
    }

    return (
      <div className="Chain">
        <div className="Chain-header">
          <Tile icon={blockIcon} title="Best Block">#{formatNumber(best)}</Tile>
          <Tile icon={blockTimeIcon} title="Avgerage Time">{ blockAverage == null ? '-' : secondsWithPrecision(blockAverage / 1000) }</Tile>
          <Tile icon={lastTimeIcon} title="Last Block"><Ago when={blockTimestamp} /></Tile>
          <div className={toggleClass.join(' ')}>
            <Icon src={worldIcon} alt="Toggle Map" onClick={this.toggleMap} />
          </div>
        </div>
        <div className="Chain-content-container">
          <div className="Chain-content">
          {
            display === 'table'
              ? this.renderTable()
              : this.renderMap()
          }
          </div>
        </div>
      </div>
    );
  }

  private toggleMap = () => {
    if (this.state.display === 'map') {
      this.setState({ display: 'table' });
    } else {
      this.setState({ display: 'map' });
    }
  }

  private renderMap() {
    return (
      <div className="Chain-map">
      {
        // Debug rect
        // <div style={{ position: 'absolute', background: 'rgba(0,255,0,0.5)', ...this.state.map}} />
      }
      {
        this.nodes().map((node) => {
          const location = node.location || [0, 0] as Types.NodeLocation;
          // const location = [51.4825891, -0.0164137] as Types.NodeLocation; // Greenwich
          // const location = [52.2330653, 20.921111] as Types.NodeLocation; // Warsaw
          // const location = [48.8589507, 2.2770201] as Types.NodeLocation; // Paris
          // const location = [36.7183391, -4.5193071]as Types.NodeLocation; // Malaga

          return (
            <span
              key={node.id}
              className="Chain-map-node"
              style={this.pixelPosition(location[0], location[1])}
              title={node.nodeDetails[0]}
              data-location={JSON.stringify(node.location)}
            />
          );
        })
      }
      </div>
    );
  }

  private renderTable() {
    return (
      <table className="Chain-node-list">
        <thead>
          <tr>
            <th><Icon src={nodeIcon} alt="Node" /></th>
            <th><Icon src={nodeTypeIcon} alt="Implementation" /></th>
            <th><Icon src={peersIcon} alt="Peer Count" /></th>
            <th><Icon src={transactionsIcon} alt="Transactions in Queue" /></th>
            <th><Icon src={blockIcon} alt="Block" /></th>
            <th><Icon src={blockHashIcon} alt="Block Hash" /></th>
            <th><Icon src={blockTimeIcon} alt="Block Time" /></th>
            <th><Icon src={propagationTimeIcon} alt="Block Propagation Time" /></th>
            <th><Icon src={lastTimeIcon} alt="Last Block Time" /></th>
          </tr>
        </thead>
        <tbody>
        {
          this.nodes().sort(sortNodes).map((node) => <Node key={node.id} {...node} />)
        }
        </tbody>
      </table>
    );
  }

  private nodes() {
    return Array.from(this.props.appState.nodes.values());
  }

  private pixelPosition(lat: Types.Latitude, lon: Types.Longitude): { left: number, top: number } {
    const { map } = this.state;

    const left = Math.round(((lon + 180) / 360) * map.width + map.left) - 35;
    const top = Math.round(((-lat + 90) / 180) * map.height + map.top) + 4;

    return { left, top }
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
}
