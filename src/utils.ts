/**
 * PhantomData akin to Rust, because sometimes you need to be smarter than
 * the compiler.
 */
export class PhantomData<P> { private __PHANTOM__: P }

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
 * Higher order function producing new auto-incremented `Id`s.
 */
export function idGenerator<T>(): () => Id<T> {
    let current = 0;

    return () => current++ as Id<T>;
}

/**
 * Unique type-constrained Id number.
 */
export type Id<T> = Opaque<number, T>;

interface HasId<T> {
    id: Id<T>;
}

export class IdSet<T> {
    private map: Map<Id<T>, T> = new Map();

    public add(item: T & HasId<T>) {
        this.map.set(item.id, item);
    }

    public remove(item: T & HasId<T>) {
        this.map.delete(item.id);
    }

    public get entries(): IterableIterator<T> {
        return this.map.values();
    }
}

export function* map<T, U>(iter: IterableIterator<T>, fn: (item: T) => U): IterableIterator<U> {
    for (const item of iter) yield fn(item);
}

export function* chain<T>(a: IterableIterator<T>, b: IterableIterator<T>): IterableIterator<T> {
    yield* a;
    yield* b;
}

export function* zip<T, U>(a: IterableIterator<T>, b: IterableIterator<U>): IterableIterator<[T, U]> {
    let itemA = a.next();
    let itemB = b.next();

    while (!itemA.done && !itemB.done) {
        yield [itemA.value, itemB.value];

        itemA = a.next();
        itemB = b.next();
    }
}

export function* take<T>(iter: IterableIterator<T>, n: number): IterableIterator<T> {
    for (const item of iter) {
        if (n-- === 0) return;

        yield item;
    }
}

export function skip<T>(iter: IterableIterator<T>, n: number): IterableIterator<T> {
    while (n-- !== 0 && !iter.next().done) {}

    return iter;
}

export function reduce<T, R>(iter: IterableIterator<T>, fn: (accu: R, item: T) => R, accumulator: R): R {
    for (const item of iter) accumulator = fn(accumulator, item);

    return accumulator;
}

export function join(iter: IterableIterator<{ toString: () => string }>, glue: string): string {
    const first = iter.next();

    if (first.done) return '';

    let result = first.value.toString();

    for (const item of iter) result += glue + item;

    return result;
}
