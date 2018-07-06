declare module 'iplocation' {
  namespace iplocation {
    export interface LocationData {
      as?: string;
      city?: string;
      country?: string;
      countryCode?: string;
      isp?: string;
      lat: number;
      lon: number;
      org?: string;
      query?: string;
      region?: string;
      regionName?: string;
      status: string;
      timezone?: string;
      zip?: string;
    }
  }

  function iplocation(ip: string, callback: (err: Error, result: iplocation.LocationData) => void): void;

  export = iplocation;
}
