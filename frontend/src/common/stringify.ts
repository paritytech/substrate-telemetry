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

export abstract class Stringified<T> {
  public __PHANTOM__: T;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const parse = JSON.parse as any as <T>(val: Stringified<T>) => T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const stringify = JSON.stringify as any as <T>(val: T) => Stringified<T>;
