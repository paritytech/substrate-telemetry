import { Persistent } from './';

export class PersistentObject<Data extends object> {
  private readonly inner: Persistent<Data>;

  constructor(key: string, initial: Data, onChange: (value: Data) => void) {
    this.inner = new Persistent(key, initial, onChange);
  }

  public raw(): Readonly<Data> {
    return this.inner.get();
  }

  public get<K extends keyof Data>(key: K): Data[K] {
    return this.inner.get()[key];
  }

  public set<K extends keyof Data>(key: K, value: Data[K]) {
    const data: Data = Object.assign({}, this.raw());
    data[key] = value;
    this.inner.set(data);
  }
}
