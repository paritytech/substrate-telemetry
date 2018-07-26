import { Maybe, Types } from '@dotstats/common';

export function blockAverage(blockTimes: Array<number>): Types.Milliseconds {
  let count = 0;
  let sum = 0;

  for (const time of blockTimes) {
    if (time) {
      count += 1;
      sum += time;
    }
  }

  if (count === 0) {
    return 0 as Types.Milliseconds;
  }

  return (sum / count) as Types.Milliseconds;
}
