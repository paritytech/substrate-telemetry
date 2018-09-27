const test = require('tape');
const common = require('../build/');

test('sortedInsert', (assert) => {
  const { sortedInsert } = common;
  const cmp = (a, b) => a - b;

  let mod = sortedInsert(3, [1,2,4,5], cmp);

  function assertInsert(item, into, equals) {
    sortedInsert(item, into, cmp);
    assert.same(into, equals, `Inserts ${item}`);
  }

  assertInsert(1, [2,3,4,5,6,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(2, [1,3,4,5,6,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(3, [1,2,4,5,6,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(4, [1,2,3,5,6,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(5, [1,2,3,4,6,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(6, [1,2,3,4,5,7,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(7, [1,2,3,4,5,6,8,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(8, [1,2,3,4,5,6,7,9], [1,2,3,4,5,6,7,8,9]);
  assertInsert(9, [1,2,3,4,5,6,7,8], [1,2,3,4,5,6,7,8,9]);

  assert.end();
});

test('sortedInsert fuzz', (assert) => {
  const { sortedInsert } = common;
  const cmp = (a, b) => a - b;
  const scramble = () => Math.random() - 0.5;
  const sorted = [1,2,3,4,5,6,7,8,9];

  for (let i = 0; i < 50; i++) {
    const scrambled = sorted.sort(scramble);
    const resorted = [];

    for (const item of scrambled) {
      sortedInsert(item, resorted, cmp);
    }

    assert.same(resorted, [1,2,3,4,5,6,7,8,9], `resort ${scrambled}`);
  }

  assert.end();
});

test('sortedInsert indexes', (assert) => {
  const { sortedInsert } = common;
  const cmp = (a, b) => a - b;
  const into = [];

  assert.equals(sortedInsert(5, into, cmp), 0, 'Insert 5');
  assert.same(into, [5], 'Elements check out');
  assert.equals(sortedInsert(1, into, cmp), 0, 'Insert 1');
  assert.same(into, [1,5], 'Elements check out');
  assert.equals(sortedInsert(9, into, cmp), 2, 'Insert 9');
  assert.same(into, [1,5,9], 'Elements check out');
  assert.equals(sortedInsert(3, into, cmp), 1, 'Insert 3');
  assert.same(into, [1,3,5,9], 'Elements check out');
  assert.equals(sortedInsert(7, into, cmp), 3, 'Insert 7');
  assert.same(into, [1,3,5,7,9], 'Elements check out');
  assert.equals(sortedInsert(4, into, cmp), 2, 'Insert 4');
  assert.same(into, [1,3,4,5,7,9], 'Elements check out');
  assert.equals(sortedInsert(6, into, cmp), 4, 'Insert 6');
  assert.same(into, [1,3,4,5,6,7,9], 'Elements check out');
  assert.equals(sortedInsert(2, into, cmp), 1, 'Insert 2');
  assert.same(into, [1,2,3,4,5,6,7,9], 'Elements check out');
  assert.equals(sortedInsert(8, into, cmp), 7, 'Insert 8');
  assert.same(into, [1,2,3,4,5,6,7,8,9], 'Elements check out');

  assert.end();
});

test('sortedIndexOf', (assert) => {
  const { sortedIndexOf } = common;
  const cmp = (a, b) => a - b;

  assert.equals(sortedIndexOf(1, [1,2,3,4,5,6,7,8,9], cmp), 0, 'Found 1');
  assert.equals(sortedIndexOf(2, [1,2,3,4,5,6,7,8,9], cmp), 1, 'Found 2');
  assert.equals(sortedIndexOf(3, [1,2,3,4,5,6,7,8,9], cmp), 2, 'Found 3');
  assert.equals(sortedIndexOf(4, [1,2,3,4,5,6,7,8,9], cmp), 3, 'Found 4');
  assert.equals(sortedIndexOf(5, [1,2,3,4,5,6,7,8,9], cmp), 4, 'Found 5');
  assert.equals(sortedIndexOf(6, [1,2,3,4,5,6,7,8,9], cmp), 5, 'Found 6');
  assert.equals(sortedIndexOf(7, [1,2,3,4,5,6,7,8,9], cmp), 6, 'Found 7');
  assert.equals(sortedIndexOf(8, [1,2,3,4,5,6,7,8,9], cmp), 7, 'Found 8');
  assert.equals(sortedIndexOf(9, [1,2,3,4,5,6,7,8,9], cmp), 8, 'Found 9');

  assert.equals(sortedIndexOf(0, [1,2,3,4,5,6,7,8,9], cmp), -1, 'No 0');
  assert.equals(sortedIndexOf(10, [1,2,3,4,5,6,7,8,9], cmp), -1, 'No 10');
  assert.equals(sortedIndexOf(5.5, [1,2,3,4,5,6,7,8,9], cmp), -1, 'No 5.5');

  assert.end();
});
