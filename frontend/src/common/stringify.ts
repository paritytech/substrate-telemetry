export abstract class Stringified<T> {
  public __PHANTOM__: T;
}

export const parse = (JSON.parse as any) as <T>(val: Stringified<T>) => T;
export const stringify = (JSON.stringify as any) as <T>(
  val: T
) => Stringified<T>;
