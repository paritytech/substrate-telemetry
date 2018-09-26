import * as React from 'react';
import { Types } from '@dotstats/common';
import { Chains, Chain, Ago, OfflineIndicator } from './components';
import { Connection } from './Connection';
import { PersistentObject, PersistentSet } from './persist';
import { State } from './state';

import './App.css';

export default class App extends React.Component<{}, State> {
  public state: State;
  private readonly settings: PersistentObject<State.Settings>;
  private readonly pins: PersistentSet<Types.NodeName>;
  private readonly connection: Promise<Connection>;

  constructor(props: {}) {
    super(props);

    this.settings = new PersistentObject(
      'settings',
      {
        validator: true,
        location: true,
        implementation: true,
        peers: true,
        txs: true,
        cpu: false,
        mem: false,
        blocknumber: true,
        blockhash: true,
        blocktime: true,
        blockpropagation: true,
        blocklasttime: false
      },
      (settings) => this.setState({ settings })
    );

    this.pins = new PersistentSet<Types.NodeName>('pinned_names', (pins) => {
      const { nodes } = this.state;

      for (const node of nodes.values()) {
        node.pinned = pins.has(node.nodeDetails[0]);
      }

      this.setState({ nodes, pins });
    });

    this.state = {
      status: 'offline',
      best: 0 as Types.BlockNumber,
      blockTimestamp: 0 as Types.Timestamp,
      blockAverage: null,
      timeDiff: 0 as Types.Milliseconds,
      subscribed: null,
      chains: new Map(),
      nodes: new Map(),
      settings: this.settings.raw(),
      pins: this.pins.get(),
    };

    this.connection = Connection.create(this.pins, (changes) => {
      if (changes) {
        this.setState(changes);
      }

      return this.state;
    });
  }

  public render() {
    const { chains, timeDiff, subscribed, status } = this.state;

    Ago.timeDiff = timeDiff;

    if (chains.size === 0) {
      return (
        <div className="App App-no-telemetry">
          <OfflineIndicator status={status} />
          Waiting for telemetry data...
        </div>
      );
    }

    return (
      <div className="App">
        <OfflineIndicator status={status} />
        <Chains chains={chains} subscribed={subscribed} connection={this.connection} />
        <Chain appState={this.state} settings={this.settings} pins={this.pins} />
      </div>
    );
  }

  public componentWillMount() {
    window.addEventListener('keydown', this.onKeyPress);
  }

  public componentWillUnmount() {
    window.removeEventListener('keydown', this.onKeyPress);
  }

  private onKeyPress = (event: KeyboardEvent) => {
    if (event.keyCode !== 9) { // TAB KEY
      return;
    }

    event.preventDefault();

    const { subscribed } = this.state;
    const chains = Array.from(this.state.chains.keys());

    let index = 0;

    if (subscribed) {
      index = (chains.indexOf(subscribed) + 1) % chains.length;

      // Do nothing if it's the same chain
      if (chains[index] === subscribed) {
        return;
      }
    }

    this.connection.then((connection) => {
      connection.subscribe(chains[index]);
    })
  }
}
