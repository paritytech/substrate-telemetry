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
import { usePersistent } from './utils';
import { AccountSection } from './AccountSection';
import { EndpointSection, parseEndpoint } from './EndpointSection';

import './Page.css';

export function Page() {
  const [relayInput, setRelayInput] = usePersistent('rpcEndpoint', '');
  const [parachainInput, setParachainInput] = usePersistent<string | null>(
    'nodeParachainRpc',
    null
  );
  const [accountInput, setAccountInput] = usePersistent('nodeAccount', '');

  const relay = parseEndpoint(relayInput);
  const [relayEndpoint, setRelayEndpoint] = React.useState<string | null>(null);
  const parachain = parseEndpoint(parachainInput ?? '');
  const [parachainEndpoint, setParachainEndpoint] = React.useState<
    string | null
  >(null);

  return (
    <div className="Page">
      <div className="Category">
        <h1>Metanode checker</h1>
        <p className="Intro">
          Run these checks from a machine on a different network than the node,
          not on the node itself. From the node&rsquo;s own machine or local
          network, connections can succeed even when the node can&rsquo;t be
          reached from the internet, for example when its ports aren&rsquo;t
          forwarded on the router or are blocked by a firewall. The node&rsquo;s
          RPC endpoint must be reachable from that machine too.
        </p>
        <EndpointSection
          title="Polkadot RPC Endpoint"
          placeholder="rpc.example.com"
          input={relayInput}
          onInputChange={setRelayInput}
          endpoint={relay}
          onRpcCheckUpdate={setRelayEndpoint}
        />
        {relayEndpoint && parachainInput != null ? (
          <>
            <EndpointSection
              title="Parachain RPC Endpoint"
              placeholder="parachain-rpc.example.com"
              input={parachainInput}
              onInputChange={setParachainInput}
              endpoint={parachain}
              onRpcCheckUpdate={setParachainEndpoint}
            />
            <div className="Section">
              <button
                type="button"
                className="Secondary"
                onClick={() => {
                  setParachainInput(null);
                  setParachainEndpoint(null);
                }}
              >
                Remove parachain
              </button>
            </div>
          </>
        ) : null}
        {relayEndpoint && parachainInput == null ? (
          <div className="Section">
            <button
              type="button"
              className="Secondary"
              onClick={() => setParachainInput('')}
            >
              + Add parachain
            </button>
          </div>
        ) : null}
        {relayEndpoint ? (
          <AccountSection
            input={accountInput}
            onInputChange={setAccountInput}
            relayRpc={relayEndpoint}
            parachainRpc={parachainEndpoint}
          />
        ) : null}
      </div>
    </div>
  );
}
