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

import { sortedInsert, sortedIndexOf } from '.';

describe('sortedInsert', () => {
  it('inserts a value in the correct place', () => {
    function assertInsert(item: number, into: number[], equals: number[]) {
      const cmp = (a: number, b: number) => a - b;
      sortedInsert(item, into, cmp);
      expect(into).toStrictEqual(equals);
    }

    assertInsert(1, [2, 3, 4, 5, 6, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(2, [1, 3, 4, 5, 6, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(3, [1, 2, 4, 5, 6, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(4, [1, 2, 3, 5, 6, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(5, [1, 2, 3, 4, 6, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(6, [1, 2, 3, 4, 5, 7, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(7, [1, 2, 3, 4, 5, 6, 8, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(8, [1, 2, 3, 4, 5, 6, 7, 9], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assertInsert(9, [1, 2, 3, 4, 5, 6, 7, 8], [1, 2, 3, 4, 5, 6, 7, 8, 9]);
  });

  it('fuzz tests insert as expected', () => {
    const cmp = (a: number, b: number) => a - b;
    const scramble = () => Math.random() - 0.5;
    const sorted = [1, 2, 3, 4, 5, 6, 7, 8, 9];

    for (let i = 0; i < 50; i++) {
      const scrambled = sorted.sort(scramble);
      const resorted: number[] = [];

      for (const item of scrambled) {
        sortedInsert(item, resorted, cmp);
      }

      expect(resorted).toStrictEqual([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
  });

  it('indexes', () => {
    const cmp = (a: number, b: number) => a - b;
    const into: number[] = [];

    expect(sortedInsert(5, into, cmp)).toStrictEqual(0);
    expect(into).toStrictEqual([5]);
    expect(sortedInsert(1, into, cmp)).toStrictEqual(0);
    expect(into).toStrictEqual([1, 5]);
    expect(sortedInsert(9, into, cmp)).toStrictEqual(2);
    expect(into).toStrictEqual([1, 5, 9]);
    expect(sortedInsert(3, into, cmp)).toStrictEqual(1);
    expect(into).toStrictEqual([1, 3, 5, 9]);
    expect(sortedInsert(7, into, cmp)).toStrictEqual(3);
    expect(into).toStrictEqual([1, 3, 5, 7, 9]);
    expect(sortedInsert(4, into, cmp)).toStrictEqual(2);
    expect(into).toStrictEqual([1, 3, 4, 5, 7, 9]);
    expect(sortedInsert(6, into, cmp)).toStrictEqual(4);
    expect(into).toStrictEqual([1, 3, 4, 5, 6, 7, 9]);
    expect(sortedInsert(2, into, cmp)).toStrictEqual(1);
    expect(into).toStrictEqual([1, 2, 3, 4, 5, 6, 7, 9]);
    expect(sortedInsert(8, into, cmp)).toStrictEqual(7);
    expect(into).toStrictEqual([1, 2, 3, 4, 5, 6, 7, 8, 9]);
  });

  it('sortedIndexOf', () => {
    type ValueObj = {
      value: number;
    };
    const cmp = (a: ValueObj, b: ValueObj) => a.value - b.value;
    const array: Array<ValueObj> = [];

    for (let i = 1; i <= 1000; i++) {
      array.push({ value: i >> 1 });
    }

    for (let i = 0; i < 50; i++) {
      const index = (Math.random() * 1000) | 0;
      const item = array[index];

      expect(sortedIndexOf(item, array, cmp)).toStrictEqual(
        array.indexOf(item)
      );
    }
  });
});
