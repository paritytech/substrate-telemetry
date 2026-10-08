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
import { CheckStatus } from './utils';

// One line of check output: nothing when idle, then pending, ok or error.
export function CheckResult({
  status,
  pendingMessage,
}: {
  status: CheckStatus;
  pendingMessage: string;
}) {
  switch (status.kind) {
    case 'idle':
      return null;
    case 'pending':
      return <div className="Check">{pendingMessage}</div>;
    case 'ok':
      return (
        <div className="Check Check-ok">
          {status.message}
          {status.details ? <pre>{status.details}</pre> : null}
        </div>
      );
    case 'error':
      return <div className="Check Error">{status.message}</div>;
  }
}
