import * as iplocation from 'iplocation';
import { Maybe, Types } from '@dotstats/common';

export interface Location {
  lat: Types.Latitude;
  lon: Types.Longitude;
  city: Types.City;
}

const cache = new Map<string, Location>();

export async function locate(ip: string): Promise<Maybe<Location>> {
  if (ip === '127.0.0.1') {
    return Promise.resolve({
      lat: 52.5166667 as Types.Latitude,
      lon: 13.4 as Types.Longitude,
      city: 'Berlin' as Types.City,
    });
  }

  const cached = cache.get(ip);

  if (cached) {
    return Promise.resolve(cached);
  }

  return new Promise<Maybe<Location>>((resolve, _) => {
    iplocation(ip, (err, result) => {
      if (err) {
        console.error(`Couldn't locate ${ip}`);

        return resolve(null);
      }

      const { latitude: lat, longitude: lon, city } = result;
      const location = { lat, lon, city } as Location;

      cache.set(ip, location);

      resolve(location);
    });
  })
}
