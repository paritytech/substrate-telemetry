import * as React from 'react';
import './OfflineIndicator.css';
import { Icon } from './Icon';
import { State } from '../state';
import offlineIcon from '../icons/zap.svg';
import upgradeIcon from '../icons/flame.svg';

export namespace OfflineIndicator {
  export interface Props {
    status: State['status'];
  }
}

export function OfflineIndicator(
  props: OfflineIndicator.Props
): React.ReactElement<any> | null {
  switch (props.status) {
    case 'online':
      return null;
    case 'offline':
      return (
        <div className="OfflineIndicator" title="Offline">
          <Icon src={offlineIcon} />
        </div>
      );
    case 'upgrade-requested':
      return (
        <div
          className="OfflineIndicator OfflineIndicator-upgrade"
          title="New Version Available"
        >
          <Icon src={upgradeIcon} />
        </div>
      );
  }
}
