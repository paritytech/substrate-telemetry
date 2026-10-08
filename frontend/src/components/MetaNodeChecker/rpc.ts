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

const DEFAULT_TIMEOUT_MS = 10000;

interface JsonRpcResponse {
  jsonrpc: '2.0';
  id: number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

let nextId = 1;

// Performs a single JSON-RPC call over a WebSocket connection to `endpoint`.
// Resolves with the `result` field, or rejects with an `Error` describing
// what went wrong.
export function rpcCall(
  endpoint: string,
  method: string,
  params: unknown[] = [],
  timeoutMs: number = DEFAULT_TIMEOUT_MS
): Promise<unknown> {
  const request = { jsonrpc: '2.0', id: nextId++, method, params };

  return callWs(endpoint, request, timeoutMs).then((res) => {
    if (res.error) {
      throw new Error(`RPC error ${res.error.code}: ${res.error.message}`);
    }
    return res.result;
  });
}

function callWs(
  endpoint: string,
  request: { id: number },
  timeoutMs: number
): Promise<JsonRpcResponse> {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(endpoint);
    let done = false;

    const finish = (fn: () => void) => {
      if (done) {
        return;
      }
      done = true;
      clearTimeout(timer);
      socket.close();
      fn();
    };

    const timer = setTimeout(
      () =>
        finish(() => reject(new Error(`Timed out after ${timeoutMs / 1000}s`))),
      timeoutMs
    );

    socket.onopen = () => socket.send(JSON.stringify(request));
    socket.onmessage = (event) => {
      let res: JsonRpcResponse;
      try {
        res = JSON.parse(event.data);
      } catch (err) {
        finish(() => reject(new Error('Invalid JSON in response')));
        return;
      }
      if (res.id === request.id) {
        finish(() => resolve(res));
      }
    };
    // Browsers don't expose WebSocket error details, so keep this generic.
    socket.onerror = () =>
      finish(() => reject(new Error('Could not connect to the endpoint')));
    socket.onclose = (event) =>
      finish(() =>
        reject(
          new Error(`Connection closed before a response (code ${event.code})`)
        )
      );
  });
}
