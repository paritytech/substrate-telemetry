import { Opaque } from './types';

/**
 * Unique type-constrained Id number.
 */
export type Id<T> = Opaque<number, T>;

/**
 * Higher order function producing new auto-incremented `Id`s.
 */
export function idGenerator<T>(): () => Id<T> {
    let current = 0;

    return () => current++ as Id<T>;
}

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
