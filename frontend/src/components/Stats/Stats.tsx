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
import { Ranking, Range } from '../../common/types';

import './Stats.css';

interface StatsProps {
  appState: Readonly<AppState>;
}

function displayPercentage(percent: number): string {
  return (Math.round(percent * 100) / 100).toFixed(2);
}

function generateRankingTable<T>(
  key: string,
  label: string,
  format: (value: T) => string,
  ranking: Ranking<T>
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

function identity(value: string | number): string {
  return value + '';
}

function formatMemory(value: Range): string {
  const [min, max] = value;
  if (min === 0) {
    return 'Less than ' + max + ' GB';
  }
  if (max === null) {
    return 'At least ' + min + ' GB';
  }
  return min + ' GB';
}

function formatYesNo(value: boolean): string {
  if (value) {
    return 'Yes';
  } else {
    return 'No';
  }
}

function formatScore(value: Range): string {
  const [min, max] = value;
  if (max === null) {
    return 'More than ' + (min / 100).toFixed(1) + 'x';
  }
  if (min === 0) {
    return 'Less than ' + (max / 100).toFixed(1) + 'x';
  }
  if (min <= 100 && max >= 100) {
    return 'Baseline';
  }
  return (min / 100).toFixed(1) + 'x';
}

export class Stats extends React.Component<StatsProps> {
  public render() {
    const { appState } = this.props;

    const children: React.ReactNode[] = [];
    function add<T>(
      key: string,
      label: string,
      format: (value: T) => string,
      ranking: Maybe<Ranking<T>>
    ) {
      if (ranking) {
        const child = generateRankingTable(key, label, format, ranking);
        if (child !== null) {
          children.push(child);
        }
      }
    }

    const stats = appState.chainStats;
    if (stats) {
      add('version', 'Version', identity, stats.version);
      add('target_os', 'Operating System', identity, stats.target_os);
      add('target_arch', 'CPU Architecture', identity, stats.target_arch);
      add('cpu', 'CPU', identity, stats.cpu);
      add('core_count', 'CPU Cores', identity, stats.core_count);
      add('cpu_vendor', 'CPU Vendor', identity, stats.cpu_vendor);
      add('memory', 'Memory', formatMemory, stats.memory);
      add(
        'is_virtual_machine',
        'Is Virtual Machine?',
        formatYesNo,
        stats.is_virtual_machine
      );
      add('linux_distro', 'Linux Distribution', identity, stats.linux_distro);
      add('linux_kernel', 'Linux Kernel', identity, stats.linux_kernel);
      add(
        'cpu_hashrate_score',
        'CPU Speed',
        formatScore,
        stats.cpu_hashrate_score
      );
      add(
        'memory_memcpy_score',
        'Memory Speed',
        formatScore,
        stats.memory_memcpy_score
      );
      add(
        'disk_sequential_write_score',
        'Disk Speed (sequential writes)',
        formatScore,
        stats.disk_sequential_write_score
      );
      add(
        'disk_random_write_score',
        'Disk Speed (random writes)',
        formatScore,
        stats.disk_random_write_score
      );
    }

    return <div className="Stats">{children}</div>;
  }
}
