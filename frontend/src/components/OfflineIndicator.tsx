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
import './OfflineIndicator.css';
import { Icon } from './Icon';
import { State } from '../state';
import offlineIcon from '../icons/zap.svg';
import upgradeIcon from '../icons/flame.svg';

interface OfflineIndicatorProps {
  status: State['status'];
}

export function OfflineIndicator(
  props: OfflineIndicatorProps
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
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
