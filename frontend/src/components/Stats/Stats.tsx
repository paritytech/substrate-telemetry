// Source code for the Substrate Telemetry Server.
// Copyright (C) 2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import { Maybe } from '../../common';
import { State as AppState } from '../../state';
import { Row } from '../List';
import { PersistentObject } from '../../persist';
import { Ranking } from '../../common/types';

import './Stats.css';

export namespace Stats {
  export type Display = 'list' | 'map' | 'Stats';

  export interface Props {
    appState: Readonly<AppState>;
  }
}

function displayPercentage(percent: number): string {
  return (Math.round(percent * 100) / 100).toFixed(2);
}

function generateRankingTable(
  key: string,
  label: string,
  format: (value: any) => string,
  ranking: Ranking
) {
  let total = ranking.other + ranking.unknown;
  ranking.list.forEach(([_, count]) => {
    total += count;
  });

  if (ranking.unknown === total) {
    return null;
  }

  const entries: React.ReactNode[] = [];
  ranking.list.forEach(([value, count]) => {
    const percent = displayPercentage((count / total) * 100);
    const index = entries.length;
    entries.push(
      <tr key={index}>
        <td className="Stats-percent">{percent}%</td>
        <td className="Stats-count">{count}</td>
        <td className="Stats-value">{format(value)}</td>
      </tr>
    );
  });

  if (ranking.other > 0) {
    const percent = displayPercentage((ranking.other / total) * 100);
    entries.push(
      <tr key="other">
        <td className="Stats-percent">{percent}%</td>
        <td className="Stats-count">{ranking.other}</td>
        <td className="Stats-value">Other</td>
      </tr>
    );
  }

  if (ranking.unknown > 0) {
    const percent = displayPercentage((ranking.unknown / total) * 100);
    entries.push(
      <tr key="unknown">
        <td className="Stats-percent">{percent}%</td>
        <td className="Stats-count">{ranking.unknown}</td>
        <td className="Stats-value Stats-unknown">Unknown</td>
      </tr>
    );
  }

  return (
    <div className="Stats-category" key={key}>
      <table>
        <thead>
          <tr>
            <th className="Stats-percent" />
            <th className="Stats-count" />
            <th className="Stats-value">{label}</th>
          </tr>
        </thead>
        <tbody>{entries}</tbody>
      </table>
    </div>
  );
}

function identity(value: any): string {
  return value + '';
}

function formatMemory(value: any): string {
  const [min, max] = value;
  if (min === 0) {
    return 'Less than ' + max + ' GB';
  }
  if (max === null) {
    return 'At least ' + min + ' GB';
  }
  return min + ' GB';
}

function formatYesNo(value: any): string {
  if (value) {
    return 'Yes';
  } else {
    return 'No';
  }
}

function formatScore(value: any): string {
  const [min, max] = value;
  if (min === 0) {
    return 'Less than ' + (max / 100).toFixed(1) + 'x';
  }
  if (max === null) {
    return 'More than ' + (min / 100).toFixed(1) + 'x';
  }
  if (min <= 100 && max >= 100) {
    return 'Baseline';
  }
  return (min / 100).toFixed(1) + 'x';
}

const LIST: any[] = [
  ['version', 'Version', identity],
  ['target_os', 'Operating System', identity],
  ['target_arch', 'CPU Architecture', identity],
  ['cpu', 'CPU', identity],
  ['core_count', 'CPU Cores', identity],
  ['memory', 'Memory', formatMemory],
  ['is_virtual_machine', 'Is Virtual Machine?', formatYesNo],
  ['linux_distro', 'Linux Distribution', identity],
  ['linux_kernel', 'Linux Kernel', identity],
  ['cpu_hashrate_score', 'CPU Speed', formatScore],
  ['memory_memcpy_score', 'Memory Speed', formatScore],
  [
    'disk_sequential_write_score',
    'Disk Speed (sequential writes)',
    formatScore,
  ],
  ['disk_random_write_score', 'Disk Speed (random writes)', formatScore],
];

export class Stats extends React.Component<Stats.Props, {}> {
  public render() {
    const { appState } = this.props;

    const children: React.ReactNode[] = [];
    LIST.forEach(([key, label, format]) => {
      if (appState.chainStats && appState.chainStats[key]) {
        const child = generateRankingTable(
          key,
          label,
          format,
          appState.chainStats[key]
        );
        if (child !== null) {
          children.push(child);
        }
      }
    });

    return <div className="Stats">{children}</div>;
  }
}
