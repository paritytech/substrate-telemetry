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
export function sortedInsert<T>(item: T, into: Array<T>, compare: Compare<T>): number {
  if (into.length === 0) {
    into.push(item);

    return 0;
  }

  let min = 0;
  let max = into.length - 1;

  while (min !== max) {
    const guess = (min + max) / 2 | 0;

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
export function sortedIndexOf<T>(item: T, within: Array<T>, compare: Compare<T>): number {
  if (within.length === 0) {
    return -1;
  }

  let min = 0;
  let max = within.length - 1;

  while (min !== max) {
    let guess = (min + max) / 2 | 0;
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

export namespace SortedCollection {
  export type StateRef = Opaque<number, 'SortedCollection.StateRef'>;
}

export class SortedCollection<Id, Item extends { id: Id }> {
  private readonly map = new Map<Id, Item>();
  private readonly compare: Compare<Item>;

  private list = Array<Item>();
  private changeRef = 0;

  constructor(compare: Compare<Item>) {
    this.compare = compare;
  }

  public ref(): SortedCollection.StateRef {
    return this.changeRef as SortedCollection.StateRef;
  }

  public add(item: Item) {
    this.map.set(item.id, item);
    sortedInsert(item, this.list, this.compare);

    this.changeRef += 1;
  }

  public remove(id: Id) {
    const item = this.map.get(id);

    if (!item) {
      return;
    }

    const index = sortedIndexOf(item, this.list, this.compare);
    this.list.splice(index, 1);
    this.map.delete(id);

    this.changeRef += 1;
  }

  public get(id: Id): Maybe<Item> {
    return this.map.get(id);
  }

  public sorted(): Array<Item> {
    return this.list;
  }

  public mut(id: Id, mutator: (item: Item) => void) {
    const item = this.map.get(id);

    if (!item) {
      return;
    }

    mutator(item);
  }

  public mutAndSort(id: Id, mutator: (item: Item) => void) {
    const item = this.map.get(id);

    if (!item) {
      return;
    }

    const index = sortedIndexOf(item, this.list, this.compare);

    mutator(item);

    this.list.splice(index, 1);

    const newIndex = sortedInsert(item, this.list, this.compare);

    if (newIndex !== index) {
      this.changeRef += 1;
    }
  }

  public mutEach(mutator: (item: Item) => void) {
    this.list.forEach(mutator);
  }

  public mutEachAndSort(mutator: (item: Item) => void) {
    this.list.forEach(mutator);
    this.list.sort(this.compare);
  }

  public clear() {
    this.map.clear();
    this.list = [];

    this.changeRef += 1;
  }

  public hasChangedSince(ref: SortedCollection.StateRef): boolean {
    return this.changeRef > ref;
  }
}
