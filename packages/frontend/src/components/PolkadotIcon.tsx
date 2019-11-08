// Copyright 2018 Paritytech via paritytech/oo7/polkadot-identicon
// Copyright 2018 @polkadot/ui-shared authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

// This has been converted from the original version that can be found at
//
// https://github.com/paritytech/oo7/blob/251ba2b7c45503b68eab4320c270b5afa9bccb60/packages/polkadot-identicon/src/index.jsx
import * as React from 'react';
import { blake2AsU8a, decodeAddress } from '@polkadot/util-crypto';

interface Circle {
  cx: number;
  cy: number;
  fill: string;
  r: number;
}

interface Scheme {
  freq: number;
  colors: number[];
}

const blake2 = (value: Uint8Array): Uint8Array =>
  blake2AsU8a(value, 512);

const S = 64;
const C = S / 2;
const Z = S / 64 * 5;
const ZERO = blake2(new Uint8Array(32));

const SCHEMA: Scheme[] = [
  // target
  { freq: 1, colors: [0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 1] },
  // cube
  { freq: 20, colors: [0, 1, 3, 2, 4, 3, 0, 1, 3, 2, 4, 3, 0, 1, 3, 2, 4, 3, 5] },
  // quazar
  { freq: 16, colors: [1, 2, 3, 1, 2, 4, 5, 5, 4, 1, 2, 3, 1, 2, 4, 5, 5, 4, 0] },
  // flower
  { freq: 32, colors: [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 3] },
  // cyclic
  { freq: 32, colors: [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6] },
  // vmirror
  { freq: 128, colors: [0, 1, 2, 3, 4, 5, 3, 4, 2, 0, 1, 6, 7, 8, 9, 7, 8, 6, 10] },
  // hmirror
  { freq: 128, colors: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 8, 6, 7, 5, 3, 4, 2, 11] },
];

const OUTER_CIRCLE: Circle = {
  cx: C,
  cy: C,
  r: C,
  fill: '#eee'
};

function getRotation (isSixPoint: boolean): { r: number; ro2: number; r3o4: number; ro4: number; rroot3o2: number; rroot3o4: number } {
  const r = isSixPoint
    ? (C / 8 * 5)
    : (C / 4 * 3);
  const rroot3o2 = r * Math.sqrt(3) / 2;
  const ro2 = r / 2;
  const rroot3o4 = r * Math.sqrt(3) / 4;
  const ro4 = r / 4;
  const r3o4 = r * 3 / 4;

  return { r, ro2, r3o4, ro4, rroot3o2, rroot3o4 };
}

function getCircleXY (isSixPoint: boolean): Array<[number, number]> {
  const { r, ro2, r3o4, ro4, rroot3o2, rroot3o4 } = getRotation(isSixPoint);

  return [
    [C, C - r],
    [C, C - ro2],
    [C - rroot3o4, C - r3o4],
    [C - rroot3o2, C - ro2],
    [C - rroot3o4, C - ro4],
    [C - rroot3o2, C],
    [C - rroot3o2, C + ro2],
    [C - rroot3o4, C + ro4],
    [C - rroot3o4, C + r3o4],
    [C, C + r],
    [C, C + ro2],
    [C + rroot3o4, C + r3o4],
    [C + rroot3o2, C + ro2],
    [C + rroot3o4, C + ro4],
    [C + rroot3o2, C],
    [C + rroot3o2, C - ro2],
    [C + rroot3o4, C - ro4],
    [C + rroot3o4, C - r3o4],
    [C, C]
  ];
}

function findScheme (d: number): Scheme {
  let sum = 0;
  const schema = SCHEMA.find((s): boolean => {
    sum += s.freq;

    return d < sum;
  });

  if (!schema) {
    throw new Error('Unable to find schema');
  }

  return schema;
}

function addressToId (address: string): Uint8Array {
  return blake2(decodeAddress(address)).map((x, i): number => (x + 256 - ZERO[i]) % 256);
}

function getColors (address: string): string[] {
  const total = SCHEMA.map((s): number => s.freq).reduce((a, b): number => a + b);
  const id = addressToId(address);
  const d = Math.floor((id[30] + id[31] * 256) % total);
  const rot = (id[28] % 6) * 3;
  const sat = (Math.floor(id[29] * 70 / 256 + 26) % 80) + 30;
  const scheme = findScheme(d);
  const palette = Array.from(id).map((x, i): string => {
    const b = (x + i % 28 * 58) % 256;

    if (b === 0) {
      return '#444';
    } else if (b === 255) {
      return 'transparent';
    }

    const h = Math.floor(b % 64 * 360 / 64);
    const l = [53, 15, 35, 75][Math.floor(b / 64)];

    return `hsl(${h}, ${sat}%, ${l}%)`;
  });

  return scheme.colors.map((_, i): string =>
    palette[scheme.colors[i < 18 ? (i + rot) % 18 : 18]]
  );
}

/**
 * @description Generate a array of the circles that make up an indenticon
 */
export default function generate (address: string, isSixPoint = false): Circle[] {
  const colors = getColors(address);

  return [OUTER_CIRCLE].concat(
    getCircleXY(isSixPoint).map(([cx, cy], index): Circle => ({
      cx, cy, r: Z, fill: colors[index]
    }))
  );
}

export namespace PolkadotIcon {
  export interface Props {
    account: string;
    size: number;
  }
}

export class PolkadotIcon extends React.Component<PolkadotIcon.Props, {}> {
  public render (): React.ReactNode {
    const { account, size } = this.props;

    return (
      <svg
        width={size}
        height={size}
        viewBox='0 0 64 64'
      >
        {generate(account, false).map(this.renderCircle)}
      </svg>
    );
  }

  private renderCircle = ({ cx, cy, r, fill }: Circle, key: number): React.ReactNode => {
    return (
      <circle
        key={key}
        cx={cx}
        cy={cy}
        r={r}
        fill={fill}
      />
    );
  }
}
