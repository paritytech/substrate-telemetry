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

import { Maybe, Opaque } from './helpers';

export type Compare<T> = (a: T, b: T) => number;

/**
 * Insert an item into a sorted array using binary search.
 *
 * @type   {T}                item    type
 * @param  {T}                item    to be inserted
 * @param  {Array<T>}         array   to be modified
 * @param  {(a, b) => number} compare function
 *
 * @return {number}                   insertion index
 */
export function sortedInsert<T>(
  item: T,
  into: Array<T>,
  compare: Compare<T>
): number {
  if (into.length === 0) {
    into.push(item);

    return 0;
  }

  let min = 0;
  let max = into.length - 1;

  while (min !== max) {
    const guess = ((min + max) / 2) | 0;

    if (compare(item, into[guess]) < 0) {
      max = Math.max(min, guess - 1);
    } else {
      min = Math.min(max, guess + 1);
    }
  }

  const insert = compare(item, into[min]) <= 0 ? min : min + 1;

  into.splice(insert, 0, item);

  return insert;
}

/**
 * Find an index of an element within a sorted array. This should be substantially
 * faster than `indexOf` for large arrays.
 *
 * @type  {T}                item    type
 * @param {T}                item    to find
 * @param {Array<T>}         array   to look through
 * @param {(a, b) => number} compare function
 *
 * @return {number}                  index of the element, `-1` if not found
 */
export function sortedIndexOf<T>(
  item: T,
  within: Array<T>,
  compare: Compare<T>
): number {
  if (within.length === 0) {
    return -1;
  }

  let min = 0;
  let max = within.length - 1;

  while (min !== max) {
    const guess = ((min + max) / 2) | 0;
    const other = within[guess];

    if (item === other) {
      return guess;
    }

    const result = compare(item, other);

    if (result < 0) {
      max = Math.max(min, guess - 1);
    } else if (result > 0) {
      min = Math.min(max, guess + 1);
    } else {
      // Equal sort value, but different reference, do value search from min
      return within.indexOf(item, min);
    }
  }

  if (item === within[min]) {
    return min;
  }

  return -1;
}

type StateRef = Opaque<number, 'SortedCollection.StateRef'>;

interface Focus {
  start: number;
  end: number;
}

export class SortedCollection<Item extends { id: number }> {
  // Comparator function used to sort the collection
  private compare: Compare<Item>;
  // Mapping item `id` to the `Item`, this uses array as a structure with
  // the assumption that `id`s provided are increments from `0`, and that
  // vacant `id`s will be re-used in the future.
  private map = Array<Maybe<Item>>();
  // Actual sorted list of `Item`s.
  private list = Array<Item>();
  // Internal tracker for changes, this number increments whenever the
  // order of the **focused** elements in the collection changes
  private changeRef = 0;
  // Marks the range of indicies that are focused for tracking.
  // **Note:** `start` is inclusive, while `end` is exclusive (much like
  // `Array.slice()`).
  private focus: Focus = { start: 0, end: 0 };

  constructor(compare: Compare<Item>) {
    this.compare = compare;
  }

  public setComparator(compare: Compare<Item>) {
    this.compare = compare;
    this.list = this.map.filter((item) => item != null) as Item[];
    this.list.sort(compare);
    this.changeRef += 1;
  }

  public add(item: Item) {
    if (this.map.length <= item.id) {
      // Grow map if item.id would be out of scope
      this.map = this.map.concat(
        Array<Maybe<Item>>(Math.max(10, 1 + item.id - this.map.length))
      );
    }

    // Remove old item if overriding
    this.remove(item.id);

    this.map[item.id] = item;

    const index = sortedInsert(item, this.list, this.compare);

    if (index < this.focus.end) {
      this.changeRef += 1;
    }
  }

  public remove(id: number) {
    const item = this.map[id];

    if (!item) {
      return;
    }

    const index = sortedIndexOf(item, this.list, this.compare);
    this.list.splice(index, 1);
    this.map[id] = null;

    if (index < this.focus.end) {
      this.changeRef += 1;
    }
  }

  public get(id: number): Maybe<Item> {
    return this.map[id];
  }

  public sorted(): Array<Item> {
    return this.list;
  }

  public mut(id: number, mutator: (item: Item) => void) {
    const item = this.map[id];

    if (!item) {
      return;
    }

    const index = sortedIndexOf(item, this.list, this.compare);

    mutator(item);

    if (index >= this.focus.start && index < this.focus.end) {
      this.changeRef += 1;
    }
  }

  public mutAndSort(id: number, mutator: (item: Item) => void) {
    const item = this.map[id];

    if (!item) {
      return;
    }

    const index = sortedIndexOf(item, this.list, this.compare);

    mutator(item);

    this.list.splice(index, 1);

    const newIndex = sortedInsert(item, this.list, this.compare);

    if (newIndex !== index) {
      const outOfFocus =
        (index < this.focus.start && newIndex < this.focus.start) ||
        (index >= this.focus.end && newIndex >= this.focus.end);

      if (!outOfFocus) {
        this.changeRef += 1;
      }
    }
  }

  public mutAndMaybeSort(
    id: number,
    mutator: (item: Item) => void,
    sort: boolean
  ) {
    if (sort) {
      this.mutAndSort(id, mutator);
    } else {
      this.mut(id, mutator);
    }
  }

  public mutEach(mutator: (item: Item) => void) {
    this.list.forEach(mutator);
  }

  public mutEachAndSort(mutator: (item: Item) => void) {
    this.list.forEach(mutator);
    this.list.sort(this.compare);
    this.changeRef += 1;
  }

  public clear() {
    this.map = [];
    this.list = [];

    this.changeRef += 1;
  }

  // Set a new `Focus`. Any changes to the order of items within the `Focus`
  // will increment `changeRef`.
  public setFocus(start: number, end: number) {
    this.focus = { start, end };
  }

  // Get the reference to current ordering state of focused items.
  public get ref(): StateRef {
    return this.changeRef as StateRef;
  }

  // Check if order of focused items has changed since obtaining a `ref`.
  public hasChangedSince(ref: StateRef): boolean {
    return this.changeRef > ref;
  }
}
