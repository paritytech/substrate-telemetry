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
import { Connection } from '../Connection';
import { Types, Maybe } from '../common';
import { ChainData } from '../state';

import './AllChains.css';

interface AllChainsProps {
  chains: ChainData[];
  subscribed: Maybe<Types.GenesisHash>;
  connection: Promise<Connection>;
}

export function AllChains(props: AllChainsProps) {
  const { chains, subscribed, connection } = props;
  const [filterText, setFilterText] = React.useState('');
  const [sortBy, setSortBy] = React.useState(SortBy.NumberOfNodes);

  function close() {
    window.location.hash = subscribed ? `#list/${subscribed}` : '#list';
  }

  function sortByAlphabetical() {
    setSortBy(SortBy.Alphabetical);
  }

  function sortByNumberOfNodes() {
    setSortBy(SortBy.NumberOfNodes);
  }

  function updateFilterText(ev: React.FormEvent<HTMLInputElement>) {
    ev.stopPropagation();
    setFilterText(ev.currentTarget.value);
  }

  function ignoreClicks(ev: React.MouseEvent) {
    ev.stopPropagation();
  }

  function subscribeToChain(chain: ChainData) {
    return () => {
      connection.then((c) => c.subscribe(chain.genesisHash));
      close();
    };
  }

  const lowercaseFilterText = filterText.toLocaleLowerCase();
  const filteredChains = chains.filter((chain) => {
    return chain.label.toLocaleLowerCase().includes(lowercaseFilterText);
  });

  // The default sort is equal to the main display, so only sort the nodes
  // if we want to sort alphabetically:
  if (sortBy === SortBy.Alphabetical) {
    filteredChains.sort((a, b) => a.label.localeCompare(b.label));
  }

  const chainHtml =
    filteredChains.length > 0
      ? filteredChains.map((chain) => (
          <Chain
            key={chain.genesisHash}
            chain={chain}
            filterText={filterText}
            isSubscribed={subscribed === chain.genesisHash}
            onClick={subscribeToChain(chain)}
          />
        ))
      : 'No chains found';

  return (
    <div className="AllChains-overlay" onClick={close}>
      <div className="AllChains-content" onClick={ignoreClicks}>
        <div className="AllChains-controls">
          <input
            type="text"
            placeholder="Filter by chain name.."
            value={filterText}
            onChange={updateFilterText}
          />
          <SortByControl
            text="#nodes"
            isActive={sortBy === SortBy.NumberOfNodes}
            onClick={sortByNumberOfNodes}
          />
          <SortByControl
            text="A-Z"
            isActive={sortBy === SortBy.Alphabetical}
            onClick={sortByAlphabetical}
          />
        </div>
        <div className="AllChains-chains">{chainHtml}</div>
      </div>
    </div>
  );
}

type SortByControlProps = {
  text: string;
  isActive: boolean;
  onClick: () => void;
};

function SortByControl(props: SortByControlProps) {
  const className = props.isActive
    ? 'AllChains-controls-sortby AllChains-controls-sortby-active'
    : 'AllChains-controls-sortby';

  return (
    <div className={className} onClick={props.onClick}>
      {props.text}
    </div>
  );
}

type ChainProps = {
  chain: ChainData;
  filterText: string;
  isSubscribed: boolean;
  onClick: () => void;
};

function Chain({ chain, isSubscribed, onClick, filterText }: ChainProps) {
  const { label, nodeCount } = chain;

  const className = isSubscribed
    ? 'AllChains-chain AllChains-chain-selected'
    : 'AllChains-chain';

  const labelHtml = filterText ? labelWithFilterText(label, filterText) : label;

  return (
    <a key={label} className={className} onClick={onClick}>
      {labelHtml}
      <span className="AllChains-node-count" title="Node Count">
        {nodeCount}
      </span>
    </a>
  );
}

enum SortBy {
  Alphabetical,
  NumberOfNodes,
}

function labelWithFilterText(label: string, filterText: string) {
  const idx = label.toLocaleLowerCase().indexOf(filterText);
  if (idx > -1) {
    return (
      <>
        {label.slice(0, idx)}
        <span className="AllChains-chain-highlighted-text">
          {label.slice(idx, idx + filterText.length)}
        </span>
        {label.slice(idx + filterText.length)}
      </>
    );
  } else {
    return label;
  }
}
