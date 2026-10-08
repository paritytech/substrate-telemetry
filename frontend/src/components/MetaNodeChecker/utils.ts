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
import { Persistent } from '../../persist';

// The state of one check: not run yet, running, passed or failed.
export type CheckStatus =
  | { kind: 'idle' }
  | { kind: 'pending' }
  | { kind: 'ok'; message: string; details?: string }
  | { kind: 'error'; message: string };

export const IDLE: CheckStatus = { kind: 'idle' };

// A value, or an error message explaining why there isn't one.
export class Result<T> {
  private constructor(
    private readonly inner:
      | { ok: true; value: T }
      | { ok: false; error: string }
  ) {}

  public static ok<T>(value: T): Result<T> {
    return new Result<T>({ ok: true, value });
  }

  public static err<T>(error: string): Result<T> {
    return new Result<T>({ ok: false, error });
  }

  public valueOrNull(): T | null {
    return this.inner.ok ? this.inner.value : null;
  }

  public errorOrNull(): string | null {
    return this.inner.ok ? null : this.inner.error;
  }
}

// A value remembered in `localStorage` under `storageKey` (and synced across
// tabs).
export function usePersistent<T>(
  storageKey: string,
  initial: T
): [T, (value: T) => void] {
  const [stored] = React.useState(
    () => new Persistent<T>(storageKey, initial, (value) => setValue(value))
  );
  const [value, setValue] = React.useState<T>(stored.get());
  return [value, (v: T) => stored.set(v)];
}
