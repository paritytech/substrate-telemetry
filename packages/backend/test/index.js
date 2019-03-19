const test = require('tape');
const MeanList = require('../build/MeanList').default;

test('MeanList', (assert) => {
  let list = new MeanList();

  assert.same(list.get(), [], 'Inits empty');

  list.push(0);

  assert.same(list.get(), [0], 'Stores the first value');

  list.push(1);
  list.push(2);
  list.push(3);
  list.push(4);
  list.push(5);
  list.push(6);
  list.push(7);
  list.push(8);
  list.push(9);

  assert.same(list.get(), [0,1,2,3,4,5,6,7,8,9], 'Stores the first 10 values');

  list.push(10);
  list.push(11);
  list.push(12);
  list.push(13);
  list.push(14);
  list.push(15);
  list.push(16);
  list.push(17);
  list.push(18);
  list.push(19);

  assert.same(list.get(), [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19], 'Stores the first 20 values');

  list.push(20);

  assert.same(list.get(), [0.5,2.5,4.5,6.5,8.5,10.5,12.5,14.5,16.5,18.5], 'Squashes values on 21st entry');

  list.push(21);

  assert.same(list.get(), [0.5,2.5,4.5,6.5,8.5,10.5,12.5,14.5,16.5,18.5,20.5], 'Adds a mean on 22nd entry');

  list.push(22);

  assert.same(list.get(), [0.5,2.5,4.5,6.5,8.5,10.5,12.5,14.5,16.5,18.5,20.5], 'Keeps track of 23rd entry internally');

  list.push(23);

  assert.same(list.get(), [0.5,2.5,4.5,6.5,8.5,10.5,12.5,14.5,16.5,18.5,20.5,22.5], 'Adds a mean on 24th entry');

  list.push(24);
  list.push(25);
  list.push(26);
  list.push(27);
  list.push(28);
  list.push(29);
  list.push(30);
  list.push(31);
  list.push(32);
  list.push(33);
  list.push(34);
  list.push(35);
  list.push(36);
  list.push(37);
  list.push(38);
  list.push(39);

  assert.same(list.get(), [
     0.5,  2.5,  4.5,  6.5,  8.5, 10.5, 12.5, 14.5, 16.5, 18.5,
    20.5, 22.5, 24.5, 26.5, 28.5, 30.5, 32.5, 34.5, 36.5, 38.5
  ], 'Adds means up to 40th entry');

  list.push(40);

  assert.same(list.get(), [
     1.5,  5.5,  9.5, 13.5, 17.5, 21.5, 25.5, 29.5, 33.5, 37.5,
  ], 'Squashes values on 41st entry');

  list = new MeanList();

  for (var i = 0; i < 640; i++) {
    list.push(i);
  }

  assert.same(list.get(), [
     15.5,   47.5,  79.5, 111.5, 143.5, 175.5, 207.5, 239.5, 271.5, 303.5,
     335.5, 367.5, 399.5, 431.5, 463.5, 495.5, 527.5, 559.5, 591.5, 623.5
  ], 'Squashes values up to 32 degrees');

  for (var i = 0; i < 31; i++) {
    list.push(i);
  }

  assert.same(list.get(), [
     15.5,   47.5,  79.5, 111.5, 143.5, 175.5, 207.5, 239.5, 271.5, 303.5,
     335.5, 367.5, 399.5, 431.5, 463.5, 495.5, 527.5, 559.5, 591.5, 623.5
  ], 'Keeps track of 31 entries internally');

  list.push(31);

  assert.same(list.get(), [
       47.5,  79.5, 111.5, 143.5, 175.5, 207.5, 239.5, 271.5, 303.5, 335.5,
      367.5, 399.5, 431.5, 463.5, 495.5, 527.5, 559.5, 591.5, 623.5,  15.5
  ], 'Pushes a new mean on 32nd value');

  assert.end();
});
