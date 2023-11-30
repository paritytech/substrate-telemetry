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
import { Types, Maybe } from '../../common';
import { formatNumber, secondsWithPrecision } from '../../utils';
import { Tab, ChainDisplay } from './';
import { Tile, Ago } from '../';

import blockIcon from '../../icons/blockchain-icon.svg';
import finalizedIcon from '../../icons/distribute-icon.svg';
import blockTimeIcon from '../../icons/stopwatch-icon.svg';
import lastTimeIcon from '../../icons/timer.svg';
import listIcon from '../../icons/list-alt-regular.svg';
import worldIcon from '../../icons/location.svg';
import settingsIcon from '../../icons/settings.svg';
import statsIcon from '../../icons/graph.svg';
import kasarImg from '../../assets/kasarLogo.png'
import deoxysImg from '../../assets/deoxys.png'

import { FiGithub } from "react-icons/fi";
import { PiTwitterLogoBold } from "react-icons/pi";
import { SiGoogledocs } from "react-icons/si";
import { MdOutlineContactSupport } from "react-icons/md";

import './Header.css';

interface HeaderProps {
  best: Types.BlockNumber;
  finalized: Types.BlockNumber;
  blockTimestamp: Types.Timestamp;
  blockAverage: Maybe<Types.Milliseconds>;
  currentTab: ChainDisplay;
  setDisplay: (display: ChainDisplay) => void;
}

type ButtonId = 'nodes' | 'map' | 'stats' | 'params';

export class Header extends React.Component<HeaderProps, { pressedButton: ButtonId | null }> {
  public shouldComponentUpdate(nextProps: HeaderProps, nextState: any) {
    return (
      this.props.best !== nextProps.best ||
      this.props.finalized !== nextProps.finalized ||
      this.props.blockTimestamp !== nextProps.blockTimestamp ||
      this.props.blockAverage !== nextProps.blockAverage ||
      this.props.currentTab !== nextProps.currentTab ||
      this.state.pressedButton !== nextState.pressedButton // Add this line
    );
  }

  state = {
    pressedButton: 'nodes' as ButtonId, // Tracks the button currently pressed
  };

  // Event handler for button clicks
  handleButtonClick = (buttonId: ButtonId) => {
    console.log('Button clicked:', buttonId); // Add this line to check if method is called
    this.setState({ pressedButton: buttonId });
  };
  public render() {
    const { best, finalized, blockTimestamp, blockAverage } = this.props;
    const { currentTab, setDisplay } = this.props;

    console.log(this.state.pressedButton)
    return (
      <div className="Header">
        <div className="Header-top-row">
          <img
            src={deoxysImg}
            alt="Deoxys"
            className="ImageIcon"
          />
          <div className="Row-icons">
            <button className="button-outline" onClick={() => window.open('https://github.com/KasarLabs/deoxys')}>
              Github
            </button>
            <button className="button-outline" onClick={() => window.open('https://twitter.com/kasarlabs')}>
              Twitter
            </button>
            <button className="button-outline" onClick={() => window.open('https://deoxys-docs.kasar.io')}>
              Docs
            </button>
            <button className="button-outline" onClick={() => window.open('https://t.me/kasarlabs')}>
              Support
            </button>
            {/* <FiGithub onClick={() => window.open('https://github.com/KasarLabs/deoxys')} size={30} />
            <PiTwitterLogoBold onClick={() => window.open('https://twitter.com/kasarlabs')} size={30} />
            <SiGoogledocs onClick={() => window.open('https://deoxys-docs.kasar.io')} size={30} />
            <MdOutlineContactSupport onClick={() => window.open('https://t.me/kasarlabs')} size={30} /> */}
          </div>
        </div>
        <div className="Header-row-first">

          <div className="Row-tiles">
            <Tile icon={blockIcon} title="Best Block">
              #{formatNumber(best)}
            </Tile>
            <Tile icon={finalizedIcon} title="Finalized Block">
              #{formatNumber(finalized)}
            </Tile>
            <Tile icon={lastTimeIcon} title="Average Time">
              {blockAverage == null
                ? '-'
                : secondsWithPrecision(blockAverage / 1000)}
            </Tile>
            <Tile icon={blockTimeIcon} title="Last Block">
              <Ago when={blockTimestamp} />
            </Tile>
          </div>

        </div>
        <div className="Header-row-second">
          {/* <button className={`button-outline ${this.state.pressedButton === 'nodes' ? 'pressed' : ''}`}
            onClick={() => this.handleButtonClick('nodes')}>
            Nodes
          </button>
          <button className={`button-outline ${this.state.pressedButton === 'map' ? 'pressed' : ''}`}
            onClick={() => this.handleButtonClick('map')}>
            Map
          </button>
          <button className={`button-outline ${this.state.pressedButton === 'stats' ? 'pressed' : ''}`}
            onClick={() => this.handleButtonClick('stats')}>
            Stats
          </button>
          <button className={`button-outline ${this.state.pressedButton === 'params' ? 'pressed' : ''}`}
            onClick={() => this.handleButtonClick('params')}>
            Params
          </button> */}

          <Tab
            text="Nodes"
            label="node"
            display="list"
            tab="node"
            current={currentTab}
            setDisplay={setDisplay}
          />
          <Tab
            text="Map"
            label="Map"
            display="map"
            tab="map"
            current={currentTab}
            setDisplay={setDisplay}
          />
          <Tab
            text="Stats"
            label="Stats"
            display="stats"
            tab="stats"
            current={currentTab}
            setDisplay={setDisplay}
          />
          <Tab
            text="Settings"
            label="Settings"
            display="settings"
            tab="settings"
            current={currentTab}
            setDisplay={setDisplay}
          />
        </div>
      </div>
    );
  }
}
