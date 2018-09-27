import { Milliseconds, Timestamp } from './types';

/**
 * PhantomData akin to Rust, because sometimes you need to be smarter than
 * the compiler.
 */
export abstract class PhantomData<P> { public __PHANTOM__: P }

/**
 * Opaque type, similar to `opaque type` in Flow, or new types in Rust/C.
 * These should be produced only by manually casting `t as Opaque<T, P>`.
 *
 * `P` can be anything as it's never actually used. Using strings is okay:
 *
 * ```
 * type MyType = Opaque<number, 'MyType'>;
 * ```
 */
export type Opaque<T, P> = T & PhantomData<P>;

/**
 * Just a readable shorthand for null-ish-able types, akin to `T?` in Flow.
 */
export type Maybe<T> = T | null | undefined;

/**
 * Asynchronous sleep
 */
export function sleep(time: Milliseconds): Promise<void> {
  return new Promise<void>((resolve, _reject) => {
    setTimeout(() => resolve(), time);
  });
}

export const timestamp = Date.now as () => Timestamp;

export function noop() {}

/**
 * Keep track of last N numbers pushed onto internal stack.
 * Provides means to get an average of said numbers.
 */
export class NumStats<T extends number> {
  private readonly stack: Array<T>;
  private readonly history: number;
  private index = 0;

  constructor(history: number) {
    if (history < 1) {
      throw new Error('Must track at least one number');
    }

    this.history = history;
    this.stack = new Array(history);
  }

  public push(val: T) {
    this.stack[this.index++ % this.history] = val;
  }

  /**
   * Get average value of all values on the stack.
   *
   * @return {T} average value
   */
  public average(): T {
    if (this.index === 0) {
      return 0 as T;
    }

    const list = this.nonEmpty();
    let sum = 0;

    for (const n of list as Array<number>) {
      sum += n;
    }

    return (sum / list.length) as T;
  }

  /**
   * Get average value of all values of the stack after filtering
   * out a number of highest and lowest values
   *
   * @param  {number} extremes number of high/low values to ignore
   * @return {T}               average value
   */
  public averageWithoutExtremes(extremes: number): T {
    if (this.index === 0) {
      return 0 as T;
    }

    const list = this.nonEmpty();
    const count = list.length - extremes * 2;

    if (count < 1) {
      // Not enough entries to remove desired number of extremes,
      // fall back to regular average
      return this.average();
    }

    let sum = 0;

    for (const n of list.sort((a, b) => a - b).slice(extremes, -extremes)) {
      sum += n;
    }

    return (sum / count) as T;
  }

  private nonEmpty(): Readonly<Array<number>> {
    return this.index < this.history ? this.stack.slice(0, this.index) : this.stack;
  }
}

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
export function sortedInsert<T>(item: T, into: Array<T>, compare: (a: T, b: T) => number): number {
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

  let insert = compare(item, into[min]) <= 0 ? min : min + 1;

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
export function sortedIndexOf<T>(item:T, within: Array<T>, compare: (a: T, b: T) => number): number {
  if (within.length === 0) {
    return -1;
  }

  let min = 0;
  let max = within.length - 1;

  while (min !== max) {
    const guess = (min + max) / 2 | 0;
    const other = within[guess];

    if (item === other) {
      return guess;
    }

    if (compare(item, other) < 0) {
      max = Math.max(min, guess - 1);
    } else {
      min = Math.min(max, guess + 1);
    }
  }

  if (item === within[min]) {
    return min;
  }

  return -1;
}
