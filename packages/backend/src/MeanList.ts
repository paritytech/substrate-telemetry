import { Maybe, Types, timestamp } from '@dotstats/common';

export class MeanList<T extends number> {
  private periodIndex = 0;
  private period = Array<T>(32).fill(0 as T);
  private meanIndex = 0;
  private means = Array<T>(20).fill(0 as T);
  private ticksPerMean = 1;

  public push(val: T) {
    this.period[this.periodIndex++] = val;

    if (this.periodIndex === this.ticksPerMean) {
      this.pushMean();
    }
  }

  public get(): Array<T> {
    if (this.meanIndex === 20) {
      return this.means;
    } else {
      return this.means.slice(0, this.meanIndex);
    }
  }

  private pushMean() {
    let sum = 0;

    for (let i = 0; i < this.periodIndex; i++) {
      sum += this.period[i] as number;
    }

    const mean = (sum / this.periodIndex) as T;

    if (this.meanIndex === 20) {
      if (this.ticksPerMean === 32) {
        this.means.copyWithin(0, 1);
        this.means[20] = mean;
      } else {
        this.squashMeans();
        this.means[this.meanIndex++] = mean;
      }
    } else {
      this.means[this.meanIndex++] = mean;
    }

    this.periodIndex = 0;
  }

  private squashMeans() {
    this.ticksPerMean *= 2;

    const means = this.means;

    for (let i = 0; i < 10; i++) {
      let i2 = i * 2;
      means[i] = (((means[i2] as number) + (means[i2 + 1] as number)) / 2) as T;
    }

    this.meanIndex = 10;
  }
}
