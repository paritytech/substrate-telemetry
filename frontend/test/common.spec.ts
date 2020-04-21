import * as test from 'tape';
import { sortedInsert, sortedIndexOf } from '../src/common';
test('sortedInsert', (assert) => {
  const cmp = (a: number, b: number) => a - b;

  let _mod = sortedInsert(3, [1, 2, 4, 5], cmp);

  function assertInsert(item: number, into: number[], equals: number[]) {
    sortedInsert(item, into, cmp);
    assert.same(into, equals, `Inserts ${item}`);
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

  assert.end();
});

test('sortedInsert fuzz', (assert) => {
  const cmp = (a: number, b: number) => a - b;
  const scramble = () => Math.random() - 0.5;
  const sorted = [1, 2, 3, 4, 5, 6, 7, 8, 9];

  for (let i = 0; i < 50; i++) {
    const scrambled = sorted.sort(scramble);
    const resorted: number[] = [];

    for (const item of scrambled) {
      sortedInsert(item, resorted, cmp);
    }

    assert.same(resorted, [1, 2, 3, 4, 5, 6, 7, 8, 9], `resort ${scrambled}`);
  }

  assert.end();
});

test('sortedInsert indexes', (assert) => {
  const cmp = (a: number, b: number) => a - b;
  const into:number[] = [];

  assert.equals(sortedInsert(5, into, cmp), 0, 'Insert 5');
  assert.same(into, [5], 'Elements check out');
  assert.equals(sortedInsert(1, into, cmp), 0, 'Insert 1');
  assert.same(into, [1, 5], 'Elements check out');
  assert.equals(sortedInsert(9, into, cmp), 2, 'Insert 9');
  assert.same(into, [1, 5, 9], 'Elements check out');
  assert.equals(sortedInsert(3, into, cmp), 1, 'Insert 3');
  assert.same(into, [1, 3, 5, 9], 'Elements check out');
  assert.equals(sortedInsert(7, into, cmp), 3, 'Insert 7');
  assert.same(into, [1, 3, 5, 7, 9], 'Elements check out');
  assert.equals(sortedInsert(4, into, cmp), 2, 'Insert 4');
  assert.same(into, [1, 3, 4, 5, 7, 9], 'Elements check out');
  assert.equals(sortedInsert(6, into, cmp), 4, 'Insert 6');
  assert.same(into, [1, 3, 4, 5, 6, 7, 9], 'Elements check out');
  assert.equals(sortedInsert(2, into, cmp), 1, 'Insert 2');
  assert.same(into, [1, 2, 3, 4, 5, 6, 7, 9], 'Elements check out');
  assert.equals(sortedInsert(8, into, cmp), 7, 'Insert 8');
  assert.same(into, [1, 2, 3, 4, 5, 6, 7, 8, 9], 'Elements check out');

  assert.end();
});

type ValueObj = {
  value: number;
};

test('sortedIndexOf', (assert) => {
  const cmp = (a: ValueObj, b: ValueObj) => a.value - b.value;
  const array: Array<ValueObj> = [];

  for (let i = 1; i <= 1000; i++) {
    array.push({ value: i >> 1 });
  }

  for (let i = 0; i < 50; i++) {
    let index = (Math.random() * 1000) | 0;
    const item = array[index];

    assert.equals(
      sortedIndexOf(item, array, cmp),
      array.indexOf(item),
      `Correct for ${item.value}`
    );
  }

  assert.end();
});
