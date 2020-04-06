import { parse, Stringified, stringify, Maybe } from '../common';

export class Persistent<Data> {
  private readonly onChange: (value: Data) => void;
  private readonly key: string;
  private value: Data;

  constructor(
    key: string,
    initial: Data,
    onChange: (value: Readonly<Data>) => void
  ) {
    this.key = key;
    this.onChange = onChange;

    const stored = window.localStorage.getItem(key) as Maybe<Stringified<Data>>;

    if (stored) {
      try {
        this.value = parse(stored);
      } catch (err) {
        this.value = initial;
      }
    } else {
      this.value = initial;
    }

    window.addEventListener('storage', (event) => {
      if (event.key === this.key) {
        this.value = parse((event.newValue as any) as Stringified<Data>);

        this.onChange(this.value);
      }
    });
  }

  public get(): Readonly<Data> {
    return this.value;
  }

  public set(value: Data) {
    this.value = value;
    window.localStorage.setItem(
      this.key,
      (stringify(this.value) as any) as string
    );
    this.onChange(this.value);
  }
}
