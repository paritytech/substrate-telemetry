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
import { IDLE, CheckStatus, Result } from './utils';
import { rpcCall } from './rpc';
import { CheckResult } from './CheckResult';
import { TextInput } from './TextInput';
import { parseWebRtcMultiaddr, stunCheck, WebRtcAddr } from './webrtc';

const RPC_SYSTEM_LOCAL_LISTEN_ADDRESSES = 'system_localListenAddresses';

interface WebRtcCheck {
  target: WebRtcAddr;
  status: CheckStatus;
}

interface EndpointSectionState {
  rpcCheck: CheckStatus;
  // One STUN check per WebRTC address the node reports.
  webRtcChecks: WebRtcCheck[];
}

const INITIAL_STATE: EndpointSectionState = {
  rpcCheck: IDLE,
  webRtcChecks: [],
};

// Turns user input such as `rpc.example.com`, `ws://127.0.0.1:9944` or
// `wss://rpc.example.com` into a WebSocket URL. An explicit `ws://` or
// `wss://` scheme is kept; input without a scheme defaults to `wss://`.
function toWsUrl(input: string): Result<string> {
  const match = /^(wss?):\/\//i.exec(input);
  const scheme = match ? match[1].toLowerCase() : 'wss';
  const withoutScheme = match ? input.slice(match[0].length) : input;

  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(withoutScheme)) {
    return Result.err('Only ws:// and wss:// endpoints are supported');
  }

  try {
    const url = new URL(`${scheme}://${withoutScheme}`);
    return Result.ok(url.href);
  } catch (err) {
    return Result.err('Not a valid URL');
  }
}

// Parses an RPC endpoint input into a WebSocket URL. The value is `null` when
// the input is empty.
export function parseEndpoint(input: string): Result<string | null> {
  const trimmed = input.trim();
  return trimmed ? toWsUrl(trimmed) : Result.ok(null);
}

// All IPv4 `/udp/<port>/webrtc-direct` listen addresses that aren't
// localhost, without duplicates.
function webRtcAddresses(addrs: unknown[]): string[] {
  const matching = addrs.filter(
    (addr): addr is string =>
      typeof addr === 'string' &&
      /^\/ip4\/(?!127\.0\.0\.1\/)[^/]+\/udp\/\d+\/webrtc-direct(\/|$)/.test(
        addr
      )
  );
  return Array.from(new Set(matching));
}

interface EndpointSectionProps {
  title: string;

  input: string;
  placeholder: string;
  onInputChange: (value: string) => void;
  // The parsed input; the value is `null` when it's empty.
  endpoint: Result<string | null>;

  onRpcCheckUpdate?: (endpoint: string | null) => void;
}

export class EndpointSection extends React.Component<
  EndpointSectionProps,
  EndpointSectionState
> {
  public state = INITIAL_STATE;

  // Ignore results from a test that was superseded by a newer one, or that
  // finished after the section was removed.
  private checkId = 0;

  public componentDidUpdate(
    _prevProps: EndpointSectionProps,
    prevState: EndpointSectionState
  ) {
    const ok = this.state.rpcCheck.kind === 'ok';
    if (ok !== (prevState.rpcCheck.kind === 'ok')) {
      const endpoint = ok ? this.props.endpoint.valueOrNull() : null;
      this.props.onRpcCheckUpdate?.(endpoint);
    }
  }

  public componentWillUnmount() {
    this.checkId++;
  }

  public render() {
    const { title, placeholder, input, endpoint } = this.props;
    const url = endpoint.valueOrNull();
    const error = endpoint.errorOrNull();
    const { rpcCheck, webRtcChecks } = this.state;

    return (
      <div className="Section">
        <h2>{title}</h2>
        <form className="Form" onSubmit={this.onSubmit}>
          <TextInput
            placeholder={placeholder}
            value={input}
            onChange={this.onInputChange}
          />
          <button type="submit" disabled={url == null || this.running()}>
            Check
          </button>
        </form>
        {error ? <div className="Error">{error}</div> : null}
        {url && rpcCheck.kind === 'idle' ? (
          <div className="Check">
            Will connect to <code>{url}</code>
          </div>
        ) : null}
        <CheckResult
          status={rpcCheck}
          pendingMessage={`Calling ${RPC_SYSTEM_LOCAL_LISTEN_ADDRESSES}…`}
        />
        {webRtcChecks.map(({ target, status }) => (
          <CheckResult
            key={`${target.ip}:${target.port}`}
            status={status}
            pendingMessage={`Sending a STUN binding request to ${target.ip}:${target.port}…`}
          />
        ))}
        {rpcCheck.kind === 'ok' && webRtcChecks.length === 0 ? (
          <div className="Check Error">
            WebRTC not checked: no IPv4 WebRTC address to check.
          </div>
        ) : null}
      </div>
    );
  }

  private running(): boolean {
    const { rpcCheck, webRtcChecks } = this.state;
    return (
      rpcCheck.kind === 'pending' ||
      webRtcChecks.some((check) => check.status.kind === 'pending')
    );
  }

  private reset() {
    this.checkId++;
    this.setState(INITIAL_STATE);
  }

  private onInputChange = (value: string) => {
    this.reset();
    this.props.onInputChange(value);
  };

  private onSubmit = (ev: React.FormEvent<HTMLFormElement>) => {
    ev.preventDefault();
    const url = this.props.endpoint.valueOrNull();
    if (url != null && !this.running()) {
      this.run(url);
    }
  };

  private async run(url: string) {
    this.reset();
    const id = this.checkId;
    const current = () => id === this.checkId;

    // Step 1: ask the node for its listen addresses over RPC.
    this.setState({ rpcCheck: { kind: 'pending' } });
    let addrs: string[];
    try {
      const result = await rpcCall(url, RPC_SYSTEM_LOCAL_LISTEN_ADDRESSES);
      addrs = webRtcAddresses(Array.isArray(result) ? result : []);
    } catch (err) {
      if (current()) {
        this.setState({
          rpcCheck: {
            kind: 'error',
            message: `RPC request failed: ${(err as Error).message}`,
          },
        });
      }
      return;
    }
    if (!current()) {
      return;
    }
    this.setState({
      rpcCheck: {
        kind: 'ok',
        message: `RPC works: ${RPC_SYSTEM_LOCAL_LISTEN_ADDRESSES} responded.`,
        details: addrs.length
          ? addrs.join('\n')
          : 'The node reports no non-localhost IPv4 WebRTC address.',
      },
    });

    // Step 2: send a STUN binding request to each WebRTC address. We can't
    // tell which one is the node's public address, so we check them all.
    const targets = addrs
      .map(parseWebRtcMultiaddr)
      .filter((target): target is WebRtcAddr => target != null);
    this.setState({
      webRtcChecks: targets.map((target) => ({
        target,
        status: { kind: 'pending' },
      })),
    });
    const setStatus = (index: number, status: CheckStatus) => {
      if (current()) {
        this.setState(({ webRtcChecks }) => ({
          webRtcChecks: webRtcChecks.map((check, i) =>
            i === index ? { ...check, status } : check
          ),
        }));
      }
    };
    await Promise.all(
      targets.map(async (target, index) => {
        const where = `${target.ip}:${target.port}`;
        try {
          const { rttMs } = await stunCheck(target);
          setStatus(index, {
            kind: 'ok',
            message:
              `WebRTC is reachable at ${where}: the node answered the STUN binding request` +
              (rttMs != null ? ` (round trip ${rttMs.toFixed(1)} ms).` : '.'),
          });
        } catch (err) {
          setStatus(index, {
            kind: 'error',
            message: `WebRTC is not reachable at ${where}: ${
              (err as Error).message
            }`,
          });
        }
      })
    );
  }
}
