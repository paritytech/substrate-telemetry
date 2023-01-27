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

import { Persistent } from './';

export class PersistentSet<Item> {
  private readonly inner: Persistent<Item[]>;
  private value: Set<Item>;

  constructor(key: string, onChange: (value: Set<Item>) => void) {
    this.inner = new Persistent(key, [], (raw: Readonly<Item[]>) =>
      onChange((this.value = new Set(raw as Item[])))
    );
    this.value = new Set(this.inner.get() as Item[]);
  }

  public get(): Set<Item> {
    return this.value;
  }

  public add(item: Item) {
    this.value.add(item);
    this.inner.set(Array.from(this.value));
  }

  public delete(item: Item) {
    this.value.delete(item);
    this.inner.set(Array.from(this.value));
  }

  public clear() {
    this.inner.set([]);
  }

  public has(item: Item): boolean {
    return this.value.has(item);
  }
}
