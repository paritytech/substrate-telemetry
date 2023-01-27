// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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
import { StateSettings } from '../../state';
import { Setting } from './';
import { Row } from '../List';
import { PersistentObject } from '../../persist';

import './Settings.css';

interface SettingsProps {
  settings: PersistentObject<StateSettings>;
}

export class Settings extends React.Component<SettingsProps> {
  public render() {
    const { settings } = this.props;

    return (
      <div className="Settings">
        <div className="Settings-category">
          <h1>List View</h1>
          <h2>Visible Columns</h2>
          {Row.columns.map(({ label, icon, setting }, index) => {
            if (!setting) {
              return null;
            }

            return (
              <Setting
                key={index}
                setting={setting}
                settings={settings}
                icon={icon}
                label={label}
              />
            );
          })}
        </div>
      </div>
    );
  }
}
