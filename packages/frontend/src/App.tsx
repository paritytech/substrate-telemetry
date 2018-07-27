import * as React from 'react';
import { Types } from '@dotstats/common';
import { Chains, Chain, Ago, OfflineIndicator } from './components';
import { Connection } from './Connection';
import { State } from './state';

import './App.css';

const NODES_PINNED = 'nodesPinned';

export default class App extends React.Component<{}, State> {
  public state: State = {
    status: 'offline',
    best: 0 as Types.BlockNumber,
    blockTimestamp: 0 as Types.Timestamp,
    blockAverage: null,
    timeDiff: 0 as Types.Milliseconds,
    subscribed: null,
    chains: new Map(),
    nodes: new Map(),
    nodesPinned: {}
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
        <Chain appState={this.state} handleNodePinClick={this.handleNodePinClick} />
      </div>
    );
  }

  public componentDidMount() {
    const { nodesPinned } = this.state;
    const cachedNodesPinned = localStorage.getItem(NODES_PINNED);
    console.log('component mount with localstorage cachedNodesPinned: ', cachedNodesPinned);

    if (!nodesPinned.length && cachedNodesPinned) {
      this.setState((prevState, props) => {
        const newNodesPinned = prevState.nodesPinned;
        // override localstorage with latest component state
        const merged = {...JSON.parse(cachedNodesPinned), ...newNodesPinned};
      
        console.log('component mount setting state to (merged): ', merged);

        return {nodesPinned: merged }
      });
    }
  }

  public componentWillMount() {
    window.addEventListener('keydown', this.onKeyPress);
  }

  public componentWillUnmount() {
    window.removeEventListener('keydown', this.onKeyPress);
  }

  private handleNodePinClick: (id: Types.NodeId) => () => void = (id) => {
    return () => {
      const { nodesPinned } = this.state;

      // console.log('nodesPinned: ', nodesPinned);
      // console.log('nodesPinned.size === 0: ', nodesPinned.size === 0);
      // console.log('nodesPinned.has(id): ', nodesPinned.hasOwnProperty(id));

      // set key to true if empty map or key not exist
      if (nodesPinned.size === 0 || nodesPinned.hasOwnProperty(id) === false) {
        this.setState((prevState, props) => {
          const newNodesPinned = prevState.nodesPinned;
          newNodesPinned[id] = true;

          console.log('handle click setting localstorage to: ', newNodesPinned);

          localStorage.setItem(NODES_PINNED, JSON.stringify(newNodesPinned));

          console.log('handle click set localstorage to: ', localStorage.getItem(NODES_PINNED));

          return {nodesPinned: newNodesPinned }
        });

      // toggle if key already exists
      } else {
        this.setState((prevState, props) => {
          const newNodesPinned = prevState.nodesPinned;
          const existingNodeIdPinnedState = newNodesPinned[id];
          newNodesPinned[id] = !existingNodeIdPinnedState;

          console.log('toggle to: ', newNodesPinned);

          return {nodesPinned: newNodesPinned }
        });
      }
    }
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
