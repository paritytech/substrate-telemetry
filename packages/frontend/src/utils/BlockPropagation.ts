// buckets:
//
// 0
// 1
// 2
// 3-5
// 6-10
// 11-20
// 21-50
// 51-100
// 101-200
// 201-500
// 501-1000

// const POWERS = [
//   1,
//   10,
//   100,
//   1000,
//   10000,
//   100000,
//   1000000,
//   10000000,
//   100000000,
//   1000000000,
// ];

const THRESHOLDS = [0, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000];

export class BlockPropagation {
  private static getBucket(behind: number): number {
    if (behind > 1000) {
      return 11;
    }

    let offset = 0;
    let significant = behind;

    while (significant > 5) {
      significant = Math.ceil(significant / 10); // significant must be an integer
      offset += 3;
    }

    if (significant <= 2) {
      return significant + offset;
    } else {
      return 3 + offset;
    }
  }

  private counts = new Array<number>(THRESHOLDS.length + 1);

  // static getThreshold(bucket: number): number {
  //   let bracket = (bucket - 1) % 3;
  //   let power = (bucket - 1) / 3 | 0;

  //   const exponent = power < POWERS.len ? POWERS[power] : 10 ** power;

  //   if (bracket < 2) {
  //     return (bracket + 1) * exponent;
  //   } else {
  //     return 5 * exponent;
  //   }
  // }

  constructor() {
    this.reset();
  }

  public reset() {
    this.counts.fill(0);
  }

  public add(block: number) {
    const bucket = BlockPropagation.getBucket(block);

    this.counts[bucket] += 1;
  }

  public sub(block: number) {
    const bucket = BlockPropagation.getBucket(block);

    this.counts[bucket] -= 1;
  }

  public list(): Readonly<number[]> {
    return this.counts;
  }
}