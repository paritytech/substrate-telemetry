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
import { decodeAddress } from '@polkadot/util-crypto';
import { IDLE, Result, CheckStatus } from './utils';
import { hasSessionKeys } from './session';
import { CheckResult } from './CheckResult';
import { TextInput } from './TextInput';

// Parses an SS58 address or hex-encoded 32-byte account ID. The value is
// `null` when the input is empty.
function parseAccount(input: string): Result<Uint8Array | null> {
  const trimmed = input.trim();
  if (!trimmed) {
    return Result.ok(null);
  }
  let id: Uint8Array;
  try {
    id = decodeAddress(trimmed);
  } catch (err) {
    return Result.err('Not a valid account address');
  }
  if (id.length !== 32) {
    return Result.err('Only 32-byte account IDs are supported');
  }
  return Result.ok(id);
}

interface AccountSectionState {
  relayCheck: CheckStatus;
  parachainCheck: CheckStatus;
}

const INITIAL_STATE: AccountSectionState = {
  relayCheck: IDLE,
  parachainCheck: IDLE,
};

interface AccountSectionProps {
  input: string;
  onInputChange: (value: string) => void;

  relayRpc: string | null;
  parachainRpc: string | null;
}

// The stash account input with a Check button, followed by the results of
// checking whether the account is registered with pallet-session on the relay
// chain and, if one was added, on the parachain.
export class AccountSection extends React.Component<
  AccountSectionProps,
  AccountSectionState
> {
  public state = INITIAL_STATE;

  public componentDidUpdate(prevProps: AccountSectionProps) {
    const { relayRpc, parachainRpc } = this.props;
    if (
      relayRpc !== prevProps.relayRpc ||
      parachainRpc !== prevProps.parachainRpc
    ) {
      this.reset();
    }
  }

  public render() {
    const { input } = this.props;
    const { relayCheck, parachainCheck } = this.state;
    const accountError = parseAccount(input).errorOrNull();

    return (
      <div className="Section">
        <h2>Stash Account</h2>
        <form className="Form" onSubmit={this.onSubmit}>
          <TextInput
            placeholder="Stash account address, e.g. 1abc..."
            value={input}
            onChange={this.onInputChange}
          />
          <button type="submit" disabled={!this.canCheck()}>
            Check
          </button>
        </form>
        {accountError ? <div className="Error">{accountError}</div> : null}
        <CheckResult
          status={relayCheck}
          pendingMessage="Relay chain: reading Session.NextKeys…"
        />
        <CheckResult
          status={parachainCheck}
          pendingMessage="Parachain: reading Session.NextKeys…"
        />
      </div>
    );
  }

  private isChecking(): boolean {
    const { relayCheck, parachainCheck } = this.state;
    return relayCheck.kind === 'pending' || parachainCheck.kind === 'pending';
  }

  private canCheck(): boolean {
    if (this.isChecking()) return false;

    const { input, relayRpc } = this.props;
    return relayRpc != null && parseAccount(input).valueOrNull() != null;
  }

  private reset() {
    this.setState(INITIAL_STATE);
  }

  private onInputChange = (value: string) => {
    this.reset();
    this.props.onInputChange(value);
  };

  private setChainStatus(
    chain: 'Relay Chain' | 'Parachain',
    status: CheckStatus
  ) {
    switch (chain) {
      case 'Relay Chain':
        this.setState({ relayCheck: status });
        break;
      case 'Parachain':
        this.setState({ parachainCheck: status });
        break;
    }
  }

  // Reads Session.NextKeys on one chain and reports the result in `setStatus`.
  private async checkRegistration(
    chain: 'Relay Chain' | 'Parachain',
    url: string,
    accountId: Uint8Array
  ) {
    let newStatus: CheckStatus = { kind: 'pending' };
    this.setChainStatus(chain, newStatus);

    try {
      newStatus = (await hasSessionKeys(url, accountId))
        ? {
            kind: 'ok',
            message: `${chain}: the account is registered with pallet-session (it has session keys set).`,
          }
        : {
            kind: 'error',
            message: `${chain}: the account is not registered with pallet-session (no session keys set).`,
          };
    } catch (err) {
      newStatus = {
        kind: 'error',
        message: `${chain}: session check failed: ${(err as Error).message}`,
      };
    }

    this.setChainStatus(chain, newStatus);
  }

  private async run(
    relayUrl: string,
    parachainUrl: string | null,
    accountId: Uint8Array
  ) {
    this.reset();

    await this.checkRegistration('Relay Chain', relayUrl, accountId);
    if (parachainUrl != null) {
      await this.checkRegistration('Parachain', parachainUrl, accountId);
    }
  }

  private onSubmit = (_ev: any) => {
    const { input, relayRpc, parachainRpc } = this.props;
    const accountId = parseAccount(input).valueOrNull();
    if (this.canCheck() && relayRpc != null && accountId != null) {
      this.run(relayRpc, parachainRpc, accountId);
    }
  };
}
