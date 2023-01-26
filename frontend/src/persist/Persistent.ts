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

import { parse, Stringified, stringify, Maybe } from '../common';

export class Persistent<Data> {
  private readonly onChange: (value: Data) => void;
  private readonly key: string;
  private value: Data;

  constructor(
    key: string,
    initial: Data,
    onChange: (value: Readonly<Data>) => void
  ) {
    this.key = key;
    this.onChange = onChange;

    const stored = window.localStorage.getItem(key) as Maybe<Stringified<Data>>;

    if (stored) {
      try {
        this.value = parse(stored);
      } catch (err) {
        this.value = initial;
      }
    } else {
      this.value = initial;
    }

    window.addEventListener('storage', (event) => {
      if (event.key === this.key) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.value = parse(event.newValue as any as Stringified<Data>);

        this.onChange(this.value);
      }
    });
  }

  public get(): Readonly<Data> {
    return this.value;
  }

  public set(value: Data) {
    this.value = value;
    window.localStorage.setItem(
      this.key,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      stringify(this.value) as any as string
    );
    this.onChange(this.value);
  }
}
