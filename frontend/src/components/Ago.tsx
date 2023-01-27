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
import './Tile.css';
import { timestamp, Types } from '../common';

interface AgoProps {
  when: Types.Timestamp;
  justTime?: boolean;
}

interface AgoState {
  now: Types.Timestamp;
}

const tickers = new Map<Ago, (ts: Types.Timestamp) => void>();

function tick() {
  const now = timestamp();

  for (const ticker of tickers.values()) {
    ticker(now);
  }

  setTimeout(tick, 100);
}

tick();

export class Ago extends React.Component<AgoProps, AgoState> {
  public static timeDiff = 0 as Types.Milliseconds;

  public state: AgoState;

  private agoStr: string;

  constructor(props: AgoProps) {
    super(props);

    this.state = {
      now: (timestamp() - Ago.timeDiff) as Types.Timestamp,
    };
    this.agoStr = this.stringify(props.when, this.state.now);
  }

  public shouldComponentUpdate(nextProps: AgoProps, nextState: AgoState) {
    const nextAgoStr = this.stringify(nextProps.when, nextState.now);

    if (this.agoStr !== nextAgoStr) {
      this.agoStr = nextAgoStr;
      return true;
    }

    return false;
  }

  public componentDidMount() {
    tickers.set(this, (now) => {
      this.setState({
        now: (now - Ago.timeDiff) as Types.Timestamp,
      });
    });
  }

  public componentWillUnmount() {
    tickers.delete(this);
  }

  public render() {
    if (this.props.when === 0) {
      return <span>-</span>;
    }

    return (
      <span title={new Date(this.props.when).toUTCString()}>{this.agoStr}</span>
    );
  }

  private stringify(when: number, now: number): string {
    const ago = Math.max(now - when, 0) / 1000;

    let agoStr: string;

    if (ago < 10) {
      agoStr = `${ago.toFixed(1)}s`;
    } else if (ago < 60) {
      agoStr = `${ago | 0}s`;
    } else if (ago < 3600) {
      agoStr = `${(ago / 60) | 0}m`;
    } else if (ago < 3600 * 24) {
      agoStr = `${(ago / 3600) | 0}h`;
    } else {
      agoStr = `${(ago / (3600 * 24)) | 0}d`;
    }

    if (this.props.justTime !== true) {
      agoStr += ' ago';
    }

    return agoStr;
  }
}
