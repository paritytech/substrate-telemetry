import { Opaque } from '../common';

export namespace MultiCounter {
  export type StateRef = Opaque<number, 'MultiCounter.StateRef'>;
}

export class MultiCounter<K> {
  private map = new Map<K, number>();
  private changeRef = 0;

  public increment(key: K) {
    const count = this.map.get(key);

    if (count == null) {
      this.map.set(key, 1);
    } else {
      this.map.set(key, count + 1);
    }

    this.changeRef += 1;
  }

  public decrement(key: K) {
    const count = this.map.get(key);

    if (count == null || count <= 1) {
      this.map.delete(key);
    } else {
      this.map.set(key, count - 1);
    }

    this.changeRef += 1;
  }

  public list(): Array<[K, number]> {
    return Array.from(this.map.entries()).sort((a, b) => b[1] - a[1]);
  }

  public clear() {
    this.map.clear();
    this.changeRef += 1;
  }

  public get ref(): MultiCounter.StateRef {
    return this.changeRef as MultiCounter.StateRef;
  }

  public hasChangedSince(ref: MultiCounter.StateRef): boolean {
    return this.changeRef > ref;
  }
}
