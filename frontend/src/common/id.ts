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

import { Opaque } from './helpers';

/**
 * Unique type-constrained Id number.
 */
export type Id<T> = Opaque<number, T>;

/**
 * Higher order function producing new auto-incremented `Id`s.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function idGenerator<I extends Id<any>>(): () => I {
  let current = 0;

  return () => current++ as I;
}
