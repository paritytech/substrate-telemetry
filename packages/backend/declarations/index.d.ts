declare module 'iplocation' {
  namespace iplocation {
    export interface LocationData {
      as?: string;
      city: string;
      country?: string;
      countryCode?: string;
      isp?: string;
      latitude: number;
      longitude: number;
      org?: string;
      query?: string;
      region?: string;
      regionName?: string;
      status: string;
      timezone?: string;
      zip?: string;
    }
  }

  function iplocation(ip: string, providers: any[], callback: (err: Error, result: iplocation.LocationData) => void): void;

  export = iplocation;
}
