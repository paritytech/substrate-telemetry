import { Persistent } from './';

export class PersistentSet<Item> {
  private readonly inner: Persistent<Item[]>;
  private value: Set<Item>;

  constructor(key: string, onChange: (value: Set<Item>) => void) {
    this.inner = new Persistent(key, [], (raw: Readonly<Item[]>) =>
      onChange((this.value = new Set(raw as Item[])))
    );
    this.value = new Set(this.inner.get() as Item[]);
  }

  public get(): Set<Item> {
    return this.value;
  }

  public add(item: Item) {
    this.value.add(item);
    this.inner.set(Array.from(this.value));
  }

  public delete(item: Item) {
    this.value.delete(item);
    this.inner.set(Array.from(this.value));
  }

  public clear() {
    this.inner.set([]);
  }

  public has(item: Item): boolean {
    return this.value.has(item);
  }
}
