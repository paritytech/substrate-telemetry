import * as React from 'react';
import { Icon } from '../';
import { State } from '../../state';
import { PersistentObject } from '../../persist';

import './Setting.css';

export namespace Setting {
  export interface Props {
    icon: string;
    label: string;
    setting: keyof State.Settings;
    settings: PersistentObject<State.Settings>;
  }
}

export class Setting extends React.Component<Setting.Props, {}> {
  public render() {
    const { icon, label, setting, settings } = this.props;

    const checked = settings.get(setting);
    const className = checked ? 'Setting Setting-on' : 'Setting';

    return (
      <div className={className} onClick={this.toggle}>
        <Icon src={icon} />
        {label}
        <span className="Setting-switch">
          <span className="Setting-knob" />
        </span>
      </div>
    );
  }

  private toggle = () => {
    const { setting, settings } = this.props;

    settings.set(setting, !settings.get(setting));
  };
}
