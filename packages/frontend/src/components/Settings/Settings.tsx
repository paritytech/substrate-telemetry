import * as React from 'react';
import { Maybe } from '@dotstats/common';
import { State as AppState } from '../../state';
import { Setting } from './';
import { Row } from '../List';
import { PersistentObject } from '../../persist';

import './Settings.css';

export namespace Settings {
  export type Display = 'list' | 'map' | 'settings';

  export interface Props {
    settings: PersistentObject<AppState.Settings>;
  }

  export interface State {
    display: Display;
    filter: Maybe<string>;
  }
}

export class Settings extends React.Component<Settings.Props, {}> {
  public render() {
    const { settings } = this.props;

    return (
      <div className="Settings">
        <div className="Settings-category">
          <h1>List View</h1>
          <h2>Visible Columns</h2>
          {
            Row.columns
              .map(({ label, icon, setting }, index) => {
                if (!setting) {
                  return null;
                }

                return <Setting key={index} setting={setting} settings={settings} icon={icon} label={label} />;
              })
          }
        </div>
      </div>
    );
  }
}
