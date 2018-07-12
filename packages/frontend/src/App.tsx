import * as React from 'react';
import { Types } from '@dotstats/common';
import { Chains, Chain, Ago } from './components';
import { Connection } from './Connection';
import { State } from './state';

import './App.css';

export default class App extends React.Component<{}, State> {
  public state: State = {
    best: 0 as Types.BlockNumber,
    blockTimestamp: 0 as Types.Timestamp,
    timeDiff: 0 as Types.Milliseconds,
    subscribed: null,
    chains: new Map(),
    nodes: new Map()
  };

  private connection: Promise<Connection>;

  constructor(props: {}) {
    super(props);

    this.connection = Connection.create((changes) => {
      if (changes) {
        this.setState(changes);
      }

      return this.state;
    });
  }

  public render() {
    const { chains, timeDiff, subscribed } = this.state;

    Ago.timeDiff = timeDiff;

    if (chains.size === 0) {
      return <div className="App App-no-telemetry">Waiting for telemetry data...</div>;
    }

    return (
      <div className="App">
        <Chains chains={chains} subscribed={subscribed} connection={this.connection} />
        <Chain state={this.state} />
      </div>
    );
  }
}
