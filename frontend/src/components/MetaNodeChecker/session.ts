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

import { xxhashAsU8a } from '@polkadot/util-crypto';
import { rpcCall } from './rpc';

// Checks via RPC whether `accountId` is registered with pallet-session, i.e.
// whether `Session.NextKeys(accountId)` is set. This is what
// `session.setKeys` writes.
export async function hasSessionKeys(
  rpcUrl: string,
  accountId: Uint8Array
): Promise<boolean> {
  // Storage key for a Twox64Concat map entry:
  // twox128("Session") ++ twox128("NextKeys") ++ twox64(id) ++ id
  const key = concat(
    xxhashAsU8a('Session', 128),
    xxhashAsU8a('NextKeys', 128),
    xxhashAsU8a(accountId, 64),
    accountId
  );
  const value = await rpcCall(rpcUrl, 'state_getStorage', [toHex(key)]);
  return value != null;
}

function concat(...parts: Uint8Array[]): Uint8Array {
  const result = new Uint8Array(parts.reduce((len, p) => len + p.length, 0));
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

function toHex(bytes: Uint8Array): string {
  return (
    '0x' + Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  );
}
