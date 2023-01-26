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

interface TruncateProps {
  text: string;
  chars?: number;
}

export class Truncate extends React.Component<TruncateProps> {
  public shouldComponentUpdate(nextProps: TruncateProps): boolean {
    return this.props.text !== nextProps.text;
  }

  public render() {
    const { text, chars } = this.props;

    if (!text) {
      return '-';
    }

    if (chars != null && text.length <= chars) {
      return text;
    }

    return chars ? (
      `${text.substr(0, chars - 1)}…`
    ) : (
      <div className="Column-truncate">{text}</div>
    );
  }
}
