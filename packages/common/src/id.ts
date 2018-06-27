import { Opaque } from './helpers';

/**
 * Unique type-constrained Id number.
 */
export type Id<T> = Opaque<number, T>;

/**
 * Higher order function producing new auto-incremented `Id`s.
 */
export function idGenerator<I extends Id<any>>(): () => I {
    let current = 0;

    return () => current++ as I;
}

interface HasId<I> {
    id: I;
}

export class IdSet<I extends Id<any>, T> {
    private map: Map<I, T> = new Map();

    public add(item: T & HasId<I>) {
        this.map.set(item.id, item);
    }

    public remove(item: T & HasId<I>) {
        this.map.delete(item.id);
    }

    public entries(): IterableIterator<[I, T]> {
        return this.map.entries();
    }

    public values(): IterableIterator<T> {
        return this.map.values();
    }
}
