import { Maybe, Types, timestamp } from '@dotstats/common';

export default class MeanList<T extends number> {
  private periodCount = 0;
  private periodSum = 0;
  private meanIndex = 0;
  private means = Array<T>(20).fill(0 as T);
  private ticksPerMean = 1;

  /**
   * Push a new value, returns true if a new mean value was produced
   *
   * @param  {T}       value
   *
   * @return {boolean}
   */
  public push(val: Maybe<T>): boolean {
    if (val == null) {
      return false;
    } 

    if (this.meanIndex === 20 && this.ticksPerMean < 32) {
      this.squashMeans();
    }

    this.periodSum += val as number;
    this.periodCount += 1;

    if (this.periodCount === this.ticksPerMean) {
      this.pushMean();
      return true;
    }

    return false;
  }

  public get(): Array<T> {
    if (this.meanIndex === 20) {
      return this.means;
    } else {
      return this.means.slice(0, this.meanIndex);
    }
  }

  private pushMean() {
    const mean = (this.periodSum / this.periodCount) as T;

    if (this.meanIndex === 20 && this.ticksPerMean === 32) {
      this.means.copyWithin(0, 1);
      this.means[19] = mean;
    } else {
      this.means[this.meanIndex++] = mean;
    }

    this.periodSum = 0;
    this.periodCount = 0;
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
