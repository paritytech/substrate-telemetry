import * as React from 'react';
import { Types, SortedCollection, Maybe, Compare } from './common';
import { AllChains, Chains, Chain, Ago, OfflineIndicator } from './components';
import { Row, Column } from './components/List';
import { Connection } from './Connection';
import { Persistent, PersistentObject, PersistentSet } from './persist';
import {
  bindState,
  State,
  Update,
  Node,
  ChainData,
  comparePinnedChains,
} from './state';
import { getHashData } from './utils';
import stable from 'stable';

import './App.css';

export default class App extends React.Component<{}, {}> {
  private chainsCache: ChainData[] = [];
  // Custom state for finer control over updates
  private readonly appState: Readonly<State>;
  private readonly appUpdate: Update;
  private readonly settings: PersistentObject<State.Settings>;
  private readonly pins: PersistentSet<Types.NodeName>;
  private readonly sortBy: Persistent<Maybe<number>>;
  private readonly connection: Promise<Connection>;

  constructor(props: {}) {
    super(props);

    this.settings = new PersistentObject(
      'settings',
      {
        validator: true,
        location: true,
        implementation: true,
        networkId: false,
        peers: true,
        txs: true,
        cpu: true,
        mem: true,
        upload: false,
        download: false,
        stateCacheSize: false,
        dbCacheSize: false,
        diskRead: false,
        diskWrite: false,
        blocknumber: true,
        blockhash: true,
        blocktime: true,
        finalized: false,
        finalizedhash: false,
        blockpropagation: true,
        blocklasttime: false,
        uptime: false,
      },
      (settings) => {
        const selectedColumns = this.selectedColumns(settings);

        this.sortBy.set(null);
        this.appUpdate({ settings, selectedColumns, sortBy: null });
      }
    );

    this.pins = new PersistentSet<Types.NodeName>('pinned_names', (pins) => {
      const { nodes } = this.appState;

      nodes.mutEachAndSort((node) => node.setPinned(pins.has(node.name)));

      this.appUpdate({ nodes, pins });
    });

    this.sortBy = new Persistent<Maybe<number>>('sortBy', null, (sortBy) => {
      const compare = this.getComparator(sortBy);

      this.appState.nodes.setComparator(compare);
      this.appUpdate({ sortBy });
    });

    const { tab = '' } = getHashData();

    this.appUpdate = bindState(this, {
      status: 'offline',
      best: 0 as Types.BlockNumber,
      finalized: 0 as Types.BlockNumber,
      consensusInfo: new Array() as Types.ConsensusInfo,
      displayConsensusLoadingScreen: true,
      authorities: new Array() as Types.Authorities,
      authoritySetId: null,
      sendFinality: false,
      blockTimestamp: 0 as Types.Timestamp,
      blockAverage: null,
      timeDiff: 0 as Types.Milliseconds,
      subscribed: null,
      chains: new Map(),
      nodes: new SortedCollection(Node.compare),
      settings: this.settings.raw(),
      pins: this.pins.get(),
      sortBy: this.sortBy.get(),
      selectedColumns: this.selectedColumns(this.settings.raw()),
      tab,
    });
    this.appState = this.appUpdate({});

    const comparator = this.getComparator(this.sortBy.get());

    this.appState.nodes.setComparator(comparator);
    this.connection = Connection.create(
      this.pins,
      this.appState,
      this.appUpdate
    );

    setInterval(() => (this.chainsCache = []), 10000); // Wipe sorted chains cache every 10 seconds
  }

  public render() {
    const { timeDiff, subscribed, status, tab } = this.appState;
    const chains = this.chains();

    Ago.timeDiff = timeDiff;

    if (chains.length === 0) {
      return (
        <div className="App App-no-telemetry">
          <OfflineIndicator status={status} />
          Waiting for telemetry&hellip;
        </div>
      );
    }

    const overlay =
      tab === 'all-chains' ? (
        <AllChains
          chains={chains}
          subscribed={subscribed}
          connection={this.connection}
        />
      ) : null;

    return (
      <div className="App">
        <OfflineIndicator status={status} />
        <Chains
          chains={chains}
          subscribed={subscribed}
          connection={this.connection}
        />
        <Chain
          appState={this.appState}
          appUpdate={this.appUpdate}
          connection={this.connection}
          settings={this.settings}
          pins={this.pins}
          sortBy={this.sortBy}
        />
        {overlay}
      </div>
    );
  }

  public componentDidMount() {
    window.addEventListener('keydown', this.onKeyPress);
    window.addEventListener('hashchange', this.onHashChange);
  }

  public componentWillUnmount() {
    window.removeEventListener('keydown', this.onKeyPress);
    window.removeEventListener('hashchange', this.onHashChange);
  }

  private onKeyPress = (event: KeyboardEvent) => {
    if (event.keyCode !== 9) {
      // TAB KEY
      return;
    }

    event.preventDefault();

    const { subscribed } = this.appState;
    const chains = Array.from(this.appState.chains.keys());

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
    });
  };

  private onHashChange = () => {
    const { tab = '' } = getHashData();

    this.appUpdate({ tab });
  };

  private chains(): ChainData[] {
    if (this.chainsCache.length === this.appState.chains.size) {
      return this.chainsCache;
    }

    this.chainsCache = stable.inplace(
      Array.from(this.appState.chains.values()),
      (a, b) => {
        const pinned = comparePinnedChains(a.label, b.label);

        if (pinned !== 0) {
          return pinned;
        }

        return b.nodeCount - a.nodeCount;
      }
    );

    return this.chainsCache;
  }

  private selectedColumns(settings: State.Settings): Column[] {
    return Row.columns.filter(
      ({ setting }) => setting == null || settings[setting]
    );
  }

  private getComparator(sortBy: Maybe<number>): Compare<Node> {
    const columns = this.appState.selectedColumns;

    if (sortBy != null) {
      const [index, rev] = sortBy < 0 ? [~sortBy, -1] : [sortBy, 1];
      const column = columns[index];

      if (column != null && column.sortBy) {
        const key = column.sortBy;

        return (a, b) => {
          const aKey = key(a);
          const bKey = key(b);

          if (aKey < bKey) {
            return -1 * rev;
          } else if (aKey > bKey) {
            return 1 * rev;
          } else {
            return Node.compare(a, b);
          }
        };
      }
    }

    return Node.compare;
  }
}
