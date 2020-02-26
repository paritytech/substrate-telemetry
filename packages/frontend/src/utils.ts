import { Types, Opaque } from '@dotstats/common';

export interface Viewport {
  width: number;
  height: number;
}

export function viewport(): Viewport {
  const width = Math.max(document.documentElement.clientWidth, window.innerWidth || 0);
  const height = Math.max(document.documentElement.clientHeight, window.innerHeight || 0);

  return { width, height };
}

export function formatNumber(num: number): string {
  const input =  num.toString();

  let output = '';
  let length = input.length;

  while (length > 3) {
    output = ',' + input.substr(length - 3, 3) + output;
    length -= 3;
  }

  output = input.substr(0, length) + output;

  return output;
}

export function trimHash(hash: string, length: number): string {
  if (hash.length < length) {
    return hash;
  }

  const side = ((length - 2) / 2) | 0;

  return hash.substr(0, side) + '..' + hash.substr(-side, side);
}

export function milliOrSecond(num: Types.Milliseconds | Types.PropagationTime): string {
  if (num < 10000) {
    return `${num}ms`;
  }

  return `${(num / 1000) | 0}s`;
}

export function secondsWithPrecision(num: number): string {
  const intString = (num | 0).toString()
  const intDigits = intString.length;

  switch (intDigits) {
    case 1: return num.toFixed(3) + 's';
    case 2: return num.toFixed(2) + 's';
    case 3: return num.toFixed(1) + 's';
    default: return intString + 's';
  }
}

export interface HashData {
  tab?: string;
  chain?: Types.ChainLabel;
};

export function getHashData(): HashData {
  const { hash } = window.location;

  if (hash[0] !== '#') {
    return {};
  }

  const [tab, rawChain] = hash.substr(1).split('/');
  const chain = decodeURIComponent(rawChain) as Types.ChainLabel;

  return { tab, chain };
}

export function setHashData(val: HashData) {
  const update = Object.assign(getHashData(), val);

  const { tab = '', chain = '' } = update;

  window.location.hash = `#${tab}/${encodeURIComponent(chain)}`;
}

export namespace Stats {
  export type StateRef = Opaque<number, 'Stats.StateRef'>;
}

export class Stats<K> {
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
    this.changeRef = +1;
  }

  public ref(): Stats.StateRef {
    return this.changeRef as Stats.StateRef;
  }

  public hasChangedSince(ref: Stats.StateRef): boolean {
    return this.changeRef > ref;
  }
}