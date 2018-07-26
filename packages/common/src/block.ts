import { Milliseconds } from './types';

export function blockAverage(blockTimes: Array<number>): Milliseconds {
  let count = 0;
  let sum = 0;

  for (const time of blockTimes) {
    if (time) {
      count += 1;
      sum += time;
    }
  }

  if (count === 0) {
    return 0 as Milliseconds;
  }

  return (sum / count) as Milliseconds;
}
