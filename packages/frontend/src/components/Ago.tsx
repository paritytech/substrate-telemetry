import * as React from 'react';
import './Tile.css';
import { timestamp, Types } from '@dotstats/common';

export namespace Ago {
  export interface Props {
    when: Types.Timestamp,
    justTime?: boolean,
  }

  export interface State {
    now: Types.Timestamp,
  }
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

export namespace Ago {
  export interface State {
    now: Types.Timestamp
  }
}

export class Ago extends React.Component<Ago.Props, Ago.State> {
  public static timeDiff = 0 as Types.Milliseconds;

  public state: Ago.State;

  constructor(props: Ago.Props) {
    super(props);

    this.state = {
      now: (timestamp() - Ago.timeDiff) as Types.Timestamp
    };
  }

  public componentWillMount() {
    tickers.set(this, (now) => {
      this.setState({
        now: (now - Ago.timeDiff) as Types.Timestamp
      });
    })
  }

  public componentWillUnmount() {
    tickers.delete(this);
  }

  public render() {
    if (this.props.when === 0) {
      return <span>-</span>;
    }

    const ago = Math.max(this.state.now - this.props.when, 0) / 1000;

    let agoStr: string;

    if (ago < 10) {
      agoStr = `${ago.toFixed(1)}s`;
    } else if (ago < 60) {
      agoStr = `${ago | 0}s`;
    } else if (ago < 3600) {
      agoStr = `${ ago / 60 | 0}m`;
    } else if (ago < 3600 * 24) {
      agoStr = `${ ago / 3600 | 0}h`;
    } else {
      agoStr = `${ ago / (3600 * 24) | 0}d`;
    }

    if (this.props.justTime !== true) {
      agoStr += ' ago';
    }

    return <span title={new Date(this.props.when).toUTCString()}>{agoStr}</span>
  }
}
