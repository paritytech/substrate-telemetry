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

export function* map<T, U>(
  iter: IterableIterator<T>,
  fn: (item: T) => U
): IterableIterator<U> {
  for (const item of iter) {
    yield fn(item);
  }
}

export function* chain<T>(
  a: IterableIterator<T>,
  b: IterableIterator<T>
): IterableIterator<T> {
  yield* a;
  yield* b;
}

export function* zip<T, U>(
  a: IterableIterator<T>,
  b: IterableIterator<U>
): IterableIterator<[T, U]> {
  let itemA = a.next();
  let itemB = b.next();

  while (!itemA.done && !itemB.done) {
    yield [itemA.value, itemB.value];

    itemA = a.next();
    itemB = b.next();
  }
}

export function* take<T>(
  iter: IterableIterator<T>,
  n: number
): IterableIterator<T> {
  for (const item of iter) {
    if (n-- === 0) {
      return;
    }

    yield item;
  }
}

export function skip<T>(
  iter: IterableIterator<T>,
  n: number
): IterableIterator<T> {
  while (n-- !== 0 && !iter.next().done) {}

  return iter;
}

export function reduce<T, R>(
  iter: IterableIterator<T>,
  fn: (accu: R, item: T) => R,
  accumulator: R
): R {
  for (const item of iter) {
    accumulator = fn(accumulator, item);
  }

  return accumulator;
}

export function join(
  iter: IterableIterator<{ toString: () => string }>,
  glue: string
): string {
  const first = iter.next();

  if (first.done) {
    return '';
  }

  let result = first.value.toString();

  for (const item of iter) {
    result += glue + item;
  }

  return result;
}
